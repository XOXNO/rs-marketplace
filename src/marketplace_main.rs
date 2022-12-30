#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();
pub mod auction;
use auction::*;
pub mod admin;
pub mod common;
pub mod creator;
pub mod events;
pub mod helpers;
pub mod offers;
pub mod storage;
pub mod views;

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
const NFT_AMOUNT: u32 = 1; // Token has to be unique to be considered NFT

#[elrond_wasm::contract]
pub trait EsdtNftMarketplace:
    storage::StorageModule
    + views::ViewsModule
    + events::EventsModule
    + helpers::HelpersModule
    + offers::CustomOffersModule
    + admin::AdminModule
    + creator::CreatorModule
    + common::CommonModule
{
    #[init]
    fn init(&self, bid_cut_percentage: u64, signer: ManagedAddress) {
        self.try_set_bid_cut_percentage(bid_cut_percentage);
        self.signer().set_if_empty(&signer);
    }

    #[payable("*")]
    #[endpoint(listing)]
    #[allow(clippy::too_many_arguments)]
    fn listing(
        &self,
        min_bid: BigUint,
        max_bid: BigUint,
        deadline: u64,
        accepted_payment_token: EgldOrEsdtTokenIdentifier,
        bid: bool,
        opt_sft_max_one_per_payment: OptionalValue<bool>,
        opt_start_time: OptionalValue<u64>,
    ) -> u64 {
        require!(self.status().get(), "Global operation enabled!");

        require!(
            self.accepted_tokens().contains(&accepted_payment_token),
            "The payment token is not whitelisted!"
        );

        let (nft_type, nft_nonce, nft_amount) = self.call_value().single_esdt().into_tuple();

        require!(
            nft_amount >= BigUint::from(NFT_AMOUNT),
            "Must tranfer at least one"
        );
        let current_time = self.blockchain().get_block_timestamp();
        let start_time = match opt_start_time {
            OptionalValue::Some(st) => st,
            OptionalValue::None => current_time,
        };

        let sft_max_one_per_payment = opt_sft_max_one_per_payment
            .into_option()
            .unwrap_or_default();

        if sft_max_one_per_payment || !bid {
            require!(
                min_bid == max_bid,
                "Price must be fixed for this type of auction (min bid equal to max bid)"
            );
        }
        if !accepted_payment_token.is_egld() {
            require!(
                accepted_payment_token.is_esdt(),
                "The payment token is not valid!"
            );
        }

        let opt_max_bid = if max_bid > 0u32 {
            require!(min_bid <= max_bid, "Min bid can't higher than max bid");

            Some(&max_bid)
        } else {
            None
        };

        require!(min_bid > 0u32, "Min bid must be higher than 0!");
        require!(
            nft_nonce > 0,
            "Only Semi-Fungible and Non-Fungible tokens can be auctioned"
        );
        require!(
            deadline > current_time || deadline == 0,
            "Deadline can't be in the past"
        );
        if deadline != 0 {
            require!(
                start_time >= current_time && start_time < deadline,
                "Invalid start time"
            );
        }

        let marketplace_cut_percentage = self.bid_cut_percentage().get();
        let creator_royalties_percentage = self.get_nft_info(&nft_type, nft_nonce).royalties;

        require!(
            &marketplace_cut_percentage + &creator_royalties_percentage < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );

        let accepted_payment_nft_nonce = 0;

        let auction_id = self.last_valid_auction_id().get() + 1;
        self.last_valid_auction_id().set(&auction_id);

        let auction_type = if nft_amount > BigUint::from(NFT_AMOUNT) {
            match sft_max_one_per_payment {
                true => AuctionType::SftOnePerPayment,
                false => AuctionType::SftAll,
            }
        } else {
            match bid {
                true => AuctionType::NftBid,
                false => AuctionType::Nft,
            }
        };

        if deadline == 0 {
            require!(
                auction_type == AuctionType::Nft
                    || auction_type == AuctionType::SftOnePerPayment
                    || (auction_type == AuctionType::SftAll && &min_bid == &max_bid),
                "Deadline is mandatory for this auction type!"
            );
        }

        let auction = Auction {
            auctioned_token_type: nft_type.clone(),
            auctioned_token_nonce: nft_nonce,

            nr_auctioned_tokens: nft_amount.clone(),
            auction_type,

            payment_token_type: accepted_payment_token,
            payment_token_nonce: accepted_payment_nft_nonce,

            min_bid,
            max_bid: opt_max_bid.cloned(),
            start_time,
            deadline,
            original_owner: self.blockchain().get_caller(),
            current_bid: BigUint::zero(),
            current_winner: ManagedAddress::zero(),
            marketplace_cut_percentage,
            creator_royalties_percentage,
        };
        // Map ID with Auction Struct
        self.auction_by_id(auction_id).set(&auction);
        self.listings().insert(auction_id); // Push ID to the auctions list
                                            // Add to the owner wallet the new Auction ID
        self.listings_by_wallet(&auction.original_owner)
            .insert(auction_id);
        // Insert nonce for sale per collection
        self.token_items_for_sale(&nft_type).insert(nft_nonce);
        // Insert auction ID per token and nonce
        self.token_auction_ids(&nft_type, nft_nonce)
            .insert(auction_id);

        self.token_items_quantity_for_sale(&nft_type, nft_nonce)
            .update(|qt| *qt += &nft_amount);

        self.collections_listed().insert(nft_type);
        //Emit event for new listed token
        self.emit_auction_token_event(auction_id, auction);

        auction_id
    }

    #[payable("*")]
    #[endpoint]
    fn bid(&self, auction_id: u64, nft_type: TokenIdentifier, nft_nonce: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let (payment_token, payment_token_nonce, payment_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();

        let mut auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();

        self.common_bid_checks(
            &auction,
            &nft_type,
            nft_nonce,
            &payment_token,
            payment_token_nonce,
            &payment_amount,
        );

        require!(
            auction.auction_type == AuctionType::SftAll
                || auction.auction_type == AuctionType::NftBid,
            "Cannot bid on this type of auction!"
        );

        let mut max_bid_reached = false;
        if let Some(max_bid) = &auction.max_bid {
            require!(
                &payment_amount <= max_bid,
                "Bid must be less than or equal to the max bid!"
            );
            max_bid_reached = &payment_amount == max_bid;
        }

        // refund losing bid
        if auction.current_winner != ManagedAddress::zero() {
            self.transfer_or_save_payment(
                &auction.current_winner,
                &auction.payment_token_type,
                auction.payment_token_nonce,
                &auction.current_bid,
            );
            self.listings_bids(&auction.current_winner)
                .remove(&auction_id);
            self.emit_out_bid_event(auction_id, &auction, &caller, &payment_amount, current_time);
        }

        // update auction bid and winner
        auction.current_bid = payment_amount;
        auction.current_winner = caller;
        self.auction_by_id(auction_id).set(&auction);
        self.listings_bids(&auction.current_winner)
            .insert(auction_id);

        if max_bid_reached {
            self.end_auction_common(auction_id, &auction, current_time);
        } else {
            self.emit_bid_event(auction_id, auction, current_time);
        }
    }

    #[endpoint(endAuction)]
    fn end_auction(&self, auction_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let auction = self.try_get_auction(auction_id);
        let current_time = self.blockchain().get_block_timestamp();
        require!(
            auction.auction_type == AuctionType::SftAll
                || auction.auction_type == AuctionType::NftBid,
            "Cannot end this type of auction!"
        );
        let deadline_reached = current_time > auction.deadline;
        let max_bid_reached = if let Some(max_bid) = &auction.max_bid {
            &auction.current_bid == max_bid
        } else {
            false
        };
        if auction.deadline == 0
            && AuctionType::SftAll == auction.auction_type
            && auction.max_bid.is_some()
            && auction.min_bid == auction.max_bid.clone().unwrap()
        {
            require!(
                self.blockchain().get_caller() == auction.original_owner,
                "You are not the owner of this auction in order to withdraw it!"
            );
        }
        require!(
            deadline_reached || max_bid_reached,
            "Auction deadline has not passed or the current bid is not equal to the max bid!"
        );
        let current_time = self.blockchain().get_block_timestamp();
        self.end_auction_common(auction_id, &auction, current_time);
    }

    #[payable("*")]
    #[endpoint(buy)]
    fn buy(
        &self,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        opt_sft_buy_amount: OptionalValue<BigUint>,
    ) {
        self.common_buy(
            auction_id,
            nft_type,
            nft_nonce,
            opt_sft_buy_amount,
            OptionalValue::None,
            OptionalValue::None,
        );
    }

    #[payable("*")]
    #[endpoint(buyFor)]
    fn buy_for(
        &self,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        opt_sft_buy_amount: OptionalValue<BigUint>,
        buy_for: OptionalValue<ManagedAddress>,
        message: OptionalValue<ManagedBuffer>,
    ) {
        self.common_buy(
            auction_id,
            nft_type,
            nft_nonce,
            opt_sft_buy_amount,
            buy_for,
            message,
        );
    }

    fn common_buy(
        &self,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        opt_sft_buy_amount: OptionalValue<BigUint>,
        buy_for: OptionalValue<ManagedAddress>,
        message: OptionalValue<ManagedBuffer>,
    ) {
        require!(self.status().get(), "Global operation enabled!");
        let (payment_token, payment_token_nonce, payment_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();
        let mut auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();

        let buy_amount = match opt_sft_buy_amount {
            OptionalValue::Some(amt) => amt,
            OptionalValue::None => BigUint::from(NFT_AMOUNT),
        };

        let buyer = match &buy_for {
            OptionalValue::Some(bu) => bu,
            OptionalValue::None => &caller,
        };

        let total_value = &buy_amount * &auction.min_bid;
        self.common_bid_checks(
            &auction,
            &nft_type,
            nft_nonce,
            &payment_token,
            payment_token_nonce,
            &payment_amount,
        );

        require!(buy_amount > 0, "Must buy more than 0");

        require!(
            auction.auction_type == AuctionType::SftOnePerPayment
                || auction.auction_type == AuctionType::Nft,
            "Cannot buy for this type of auction!"
        );
        require!(
            buy_amount <= auction.nr_auctioned_tokens,
            "Not enough quantity available!"
        );
        require!(
            total_value == payment_amount,
            "Wrong amount paid, must pay equal to the selling price!"
        );

        auction.current_winner = buyer.clone();
        auction.current_bid = payment_amount;
        self.distribute_tokens(&auction, Option::Some(&buy_amount));
        auction.nr_auctioned_tokens -= &buy_amount;
        if auction.nr_auctioned_tokens == 0 {
            self.remove_auction_common(auction_id, &auction);
        } else {
            self.auction_by_id(auction_id).set(&auction);
        }
        self.update_or_remove_items_quantity(&auction, &buy_amount);

        let current_time = self.blockchain().get_block_timestamp();
        self.emit_buy_event(
            auction_id,
            auction,
            buy_amount,
            current_time,
            message,
            if buy_for.into_option().is_some() {
                OptionalValue::Some(caller)
            } else {
                OptionalValue::None
            },
        );
    }

    #[payable("*")]
    #[endpoint(bulkBuy)]
    fn bulk_buy(
        &self,
        #[payment_token] payment_token: EgldOrEsdtTokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        auction_ids: MultiValueEncoded<u64>,
    ) {
        let mut total_available = payment_amount.clone();
        for auction_id in auction_ids.into_iter() {
            if !self.does_auction_exist(auction_id) {
                continue;
            }
            let listing = self.try_get_auction(auction_id);
            require!(
                listing.auction_type == AuctionType::Nft,
                "You can bulk buy just NFTs on sell with a fixed price!"
            );
            require!(
                total_available >= listing.min_bid,
                "You do not have funds to buy all the NFTs!"
            );
            // let buy_amount = listing.min_bid.clone();
            total_available -= listing.min_bid;
            // self.buy(
            //     payment_token.clone(),
            //     payment_token_nonce,
            //     buy_amount,
            //     auction_id,
            //     listing.auctioned_token_type,
            //     listing.auctioned_token_nonce,
            //     OptionalValue::None,
            // );
        }
        if total_available > BigUint::zero() {
            self.send().direct(
                &self.blockchain().get_caller(),
                &payment_token,
                payment_token_nonce,
                &total_available,
            )
        }
    }

    #[endpoint]
    fn withdraw(&self, auction_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();
        require!(
            &auction.original_owner == &caller,
            "Only the original owner can withdraw!"
        );
        self.withdraw_auction_common(auction_id, &auction);
    }

    #[endpoint(changePrice)]
    fn change_price(&self, auction_id: u64, new_price: &BigUint) {
        require!(self.status().get(), "Global operation enabled!");
        let mut auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();

        require!(
            auction.original_owner == caller,
            "Only the original owner can change the price!"
        );
        require!(
            auction.auction_type == AuctionType::Nft
                || auction.auction_type == AuctionType::SftOnePerPayment,
            "You can not change the price of bids!"
        );

        let current_time = self.blockchain().get_block_timestamp();
        self.emit_change_price_event(auction_id, &auction, new_price, current_time);
        auction.max_bid = Some(new_price.clone());
        auction.min_bid = new_price.clone();
        self.auction_by_id(auction_id).set(auction);
    }
}
