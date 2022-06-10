#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod auction;
use auction::*;

mod events;
pub mod storage;
pub mod views;

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
const NFT_AMOUNT: u32 = 1; // Token has to be unique to be considered NFT

#[elrond_wasm::contract]
pub trait EsdtNftMarketplace:
    storage::StorageModule + views::ViewsModule + events::EventsModule
{
    #[init]
    fn init(&self, bid_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(bid_cut_percentage)
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(setCutPercentage)]
    fn set_percentage_cut(&self, new_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(new_cut_percentage)
    }

    #[payable("*")]
    #[endpoint(addRewardBalance)]
    fn add_reward_balance(
        &self,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] amount: BigUint,
    ) {
        require!(
            self.reward_ticker().get() == token,
            "This token is not used for rewards!"
        );
        self.reward_balance().update(|qt| *qt += &amount.clone());
    }

    #[only_owner]
    #[endpoint(setRewardTicker)]
    fn set_reward_ticker(&self, token: TokenIdentifier) {
        require!(
            self.reward_ticker().is_empty(),
            "The ticker was already set!"
        );
        self.reward_ticker().set(token);
    }

    #[only_owner]
    #[endpoint(setSpecialRewardAmount)]
    fn set_special_reward_amount(&self, token: TokenIdentifier, amount: BigUint) {
        self.special_reward_amount(token).set(amount);
    }

    #[only_owner]
    #[endpoint(setDefaultRewardAmount)]
    fn set_default_reward_amount(&self, amount: BigUint) {
        self.reward_amount().set(amount);
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(addBlackListWallet)]
    fn add_blacklist(&self, wallet: ManagedAddress) -> bool {
        self.blacklist_wallets().insert(wallet)
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(setAcceptedTokens)]
    fn set_accepted_tokens(&self, token: TokenIdentifier) {
        self.accepted_tokens().insert(token);
    }

    #[only_owner]
    #[endpoint(removeAcceptedTokens)]
    fn remove_accepted_tokens(&self, token: TokenIdentifier) -> bool {
        self.accepted_tokens().remove(&token)
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(addWitelistedSC)]
    fn add_whitelisted_sc(&self, sc: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&sc),
            "The address is not a smart contract!"
        );
        self.whitelisted_contracts().insert(sc.clone());
        let mut tokens = self.claimable_tokens(&sc);
        for token in tokens.iter() {
            let mut nonces = self.claimable_token_nonces(&sc, &token);
            for nonce in nonces.iter() {
                let amount_map = self.claimable_amount(&sc, &token, nonce);
                let amount = amount_map.get();
                if amount > BigUint::zero() {
                    self.send()
                        .direct(&sc, &token, nonce, &amount_map.get(), &[]);
                    amount_map.clear();
                }
            }
            nonces.clear();
        }
        tokens.clear();
    }

    #[only_owner]
    #[endpoint(removeWitelistedSC)]
    fn remove_wl_sc(&self, sc: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&sc),
            "The address is not a smart contract!"
        );
        self.whitelisted_contracts().remove(&sc);
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(setStatus)]
    fn set_status(&self, status: bool) {
        self.status().set(&status);
    }
    // endpoints

    #[payable("*")]
    #[endpoint(listing)]
    #[allow(clippy::too_many_arguments)]
    fn listing(
        &self,
        #[payment_token] nft_type: TokenIdentifier,
        #[payment_nonce] nft_nonce: u64,
        #[payment_amount] nft_amount: BigUint,
        min_bid: BigUint,
        max_bid: BigUint,
        deadline: u64,
        accepted_payment_token: TokenIdentifier,
        bid: bool,
        #[var_args] opt_sft_max_one_per_payment: OptionalValue<bool>,
        #[var_args] opt_start_time: OptionalValue<u64>,
    ) -> u64 {
        require!(self.status().get(), "Global operation enabled!");

        require!(
            self.accepted_tokens().contains(&accepted_payment_token),
            "The payment token is not whitelisted!"
        );

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
                accepted_payment_token.is_valid_esdt_identifier(),
                "The payment token is not valid!"
            );
        }

        let opt_max_bid = if max_bid > 0u32 {
            require!(min_bid <= max_bid, "Min bid can't higher than max bid");

            Some(max_bid.clone())
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
                    || (auction_type == AuctionType::SftAll && min_bid == max_bid.clone()),
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
            max_bid: opt_max_bid,
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
        self.collections_listed().insert(nft_type.clone());
        self.listings().insert(auction_id); // Push ID to the auctions list
                                            // Add to the owner wallet the new Auction ID
        self.listings_by_wallet(auction.original_owner.clone())
            .insert(auction_id.clone());
        // Insert nonce for sale per collection
        self.token_items_for_sale(nft_type.clone())
            .insert(nft_nonce);
        // Insert auction ID per token and nonce
        self.token_auction_ids(nft_type.clone(), nft_nonce.clone())
            .insert(auction_id);

        self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
            .update(|qt| *qt += &nft_amount.clone());

        //Emit event for new listed token
        self.emit_auction_token_event(auction_id, auction);

        auction_id
    }

    #[payable("*")]
    #[endpoint(sendOffer)]
    fn send_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        nft_amount: BigUint,
        deadline: u64,
    ) -> u64 {
        require!(self.status().get(), "Global operation enabled!");

        require!(
            self.accepted_tokens().contains(&payment_token),
            "The payment token is not whitelisted!"
        );
        require!(
            nft_nonce > 0,
            "Only Semi-Fungible and Non-Fungible tokens can have offers"
        );
        require!(
            nft_amount == BigUint::from(NFT_AMOUNT),
            "The quantity has to be 1!"
        );

        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();
        require!(
            !self.blacklist_wallets().contains(&caller),
            "Your address was blacklisted, all your SCAM offers are lost!"
        );
        require!(
            !self
                .check_offer_sent(
                    caller.clone(),
                    nft_type.clone(),
                    nft_nonce,
                    payment_token.clone()
                )
                .get(),
            "You already sent an offer for this NFT with the same token!"
        );
        if !payment_token.is_egld() {
            require!(
                payment_token.is_valid_esdt_identifier(),
                "The payment token is not valid!"
            );
        }

        require!(
            nft_type.is_valid_esdt_identifier(),
            "The NFT token is not valid!"
        );

        require!(deadline > current_time, "Deadline can't be in the past!");

        let marketplace_cut_percentage = self.bid_cut_percentage().get();

        let offer_id = self.last_valid_offer_id().get() + 1;
        self.last_valid_offer_id().set(&offer_id);

        let offer = Offer {
            token_type: nft_type.clone(),
            token_nonce: nft_nonce.clone(),
            quantity: nft_amount.clone(),
            payment_token_type: payment_token.clone(),
            payment_token_nonce,
            status: OfferStatus::Pending,
            price: payment_amount,
            deadline,
            timestamp: current_time,
            offer_owner: caller.clone(),
            marketplace_cut_percentage,
        };
        // Map ID with Offer Struct
        self.offer_by_id(offer_id).set(&offer);
        self.token_offers_ids(nft_type.clone(), nft_nonce)
            .insert(offer_id);
        // Push ID to the offers list
        self.offers().insert(offer_id);
        // Add to the owner wallet the new Offer ID
        self.offers_by_wallet(offer.offer_owner.clone())
            .insert(offer_id.clone());
        self.check_offer_sent(
            caller.clone(),
            nft_type.clone(),
            nft_nonce,
            payment_token.clone(),
        )
        .set(&true);
        // Emit event for new offer
        self.emit_offer_token_event(offer_id, offer);

        offer_id
    }

    #[only_owner]
    #[endpoint(deleteOffersByWallet)]
    fn delete_user_offers(&self, user: ManagedAddress) {
        let offers_root = self.offers_by_wallet(user.clone());
        if offers_root.len() > 0 {
            for offer in offers_root.iter().take(80) {
                self.internal_withdraw_offer(offer);
            }
        }
    }

    #[payable("*")]
    #[endpoint]
    fn bid(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
    ) {
        require!(self.status().get(), "Global operation enabled!");
        let mut auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();

        require!(
            auction.auction_type == AuctionType::SftAll
                || auction.auction_type == AuctionType::NftBid,
            "Cannot bid on this type of auction!"
        );
        require!(
            auction.auctioned_token_type == nft_type && auction.auctioned_token_nonce == nft_nonce,
            "Auction ID does not match the token!"
        );
        require!(
            auction.original_owner != caller,
            "Cannot bid on your own token!"
        );
        require!(
            current_time >= auction.start_time,
            "Auction hasn't started yet!"
        );
        require!(
            current_time < auction.deadline
                || (auction.deadline == 0
                    && AuctionType::SftAll == auction.auction_type
                    && auction.max_bid.is_some()
                    && auction.min_bid == auction.max_bid.clone().unwrap()),
            "Auction ended already!"
        );
        require!(
            payment_token == auction.payment_token_type,
            "Wrong token used as payment!"
        );
        require!(auction.current_winner != caller, "Can't outbid yourself!");
        require!(
            payment_amount >= auction.min_bid,
            "Bid must be higher than or equal to the min bid!"
        );
        require!(
            payment_amount > auction.current_bid,
            "Bid must be higher than the current winning bid!"
        );

        if let Some(max_bid) = auction.max_bid.as_ref() {
            require!(
                &payment_amount <= max_bid,
                "Bid must be less than or equal to the max bid!"
            );
        }

        let current_time = self.blockchain().get_block_timestamp();

        // refund losing bid
        if auction.current_winner != ManagedAddress::zero() {
            self.transfer_or_save_payment(
                &auction.current_winner,
                &auction.payment_token_type,
                auction.payment_token_nonce,
                &auction.current_bid,
                b"Trust Market refunded your bid!",
            );
            self.listings_bids(auction.current_winner.clone())
                .remove(&auction_id);
            self.emit_out_bid_event(
                auction_id,
                &auction,
                caller.clone(),
                payment_amount.clone(),
                current_time,
            );
        }

        // update auction bid and winner
        auction.payment_token_nonce = payment_token_nonce;
        auction.current_bid = payment_amount;
        auction.current_winner = caller;
        self.auction_by_id(auction_id).set(&auction);
        self.listings_bids(auction.current_winner.clone())
            .insert(auction_id);

        if let Some(max_bid) = &auction.max_bid {
            if &auction.current_bid.clone() == max_bid {
                self.buy_now_bid(auction_id);
            }
        }
        if auction.max_bid.is_none() {
            self.emit_bid_event(auction_id, auction, current_time);
        } else {
            if let Some(max_bid) = &auction.max_bid {
                if &auction.current_bid.clone() != max_bid {
                    self.emit_bid_event(auction_id, auction, current_time);
                }
            }
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
        self.distribute_tokens(&auction, None);
        self.listings_by_wallet(auction.original_owner.clone())
            .remove(&auction_id);
        self.listings_bids(auction.current_winner.clone())
            .remove(&auction_id);
        self.token_auction_ids(
            auction.auctioned_token_type.clone(),
            auction.auctioned_token_nonce.clone(),
        )
        .remove(&auction_id);
        self.listings().remove(&auction_id);
        self.auction_by_id(auction_id).clear();
        self.emit_end_auction_event(auction_id, auction, current_time);
    }

    fn buy_now_bid(&self, auction_id: u64) {
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

        require!(
            deadline_reached || max_bid_reached,
            "Auction deadline has not passed or the current bid is not equal to the max bid!"
        );
        let current_time = self.blockchain().get_block_timestamp();
        self.distribute_tokens(&auction, None);
        self.listings_by_wallet(auction.original_owner.clone())
            .remove(&auction_id);
        self.listings_bids(auction.current_winner.clone())
            .remove(&auction_id);
        self.token_auction_ids(
            auction.auctioned_token_type.clone(),
            auction.auctioned_token_nonce.clone(),
        )
        .remove(&auction_id);
        self.listings().remove(&auction_id);
        self.auction_by_id(auction_id).clear();
        self.emit_end_auction_event(auction_id, auction, current_time);
    }

    #[payable("*")]
    #[endpoint(buy)]
    fn buy(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        #[var_args] opt_sft_buy_amount: OptionalValue<BigUint>,
    ) {
        require!(self.status().get(), "Global operation enabled!");
        let mut auction = self.try_get_auction(auction_id);
        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();

        let buy_amount = match opt_sft_buy_amount {
            OptionalValue::Some(amt) => amt,
            OptionalValue::None => BigUint::from(NFT_AMOUNT),
        };

        let total_value = &buy_amount * &auction.min_bid;

        require!(buy_amount > 0, "The amount must be more than 0!");
        require!(
            payment_amount.gt(&BigUint::zero()),
            "The paid amount must be higher than 0!"
        );
        require!(
            auction.auction_type == AuctionType::SftOnePerPayment
                || auction.auction_type == AuctionType::Nft,
            "Cannot buy for this type of auction!"
        );
        require!(
            auction.auctioned_token_type == nft_type && auction.auctioned_token_nonce == nft_nonce,
            "Auction ID does not match the token!"
        );
        require!(
            auction.original_owner != caller,
            "Cannot buy your own token!"
        );
        require!(
            buy_amount <= auction.nr_auctioned_tokens,
            "Not enough quantity available!"
        );
        require!(
            payment_token == auction.payment_token_type,
            "Wrong token used as payment"
        );
        require!(
            total_value == payment_amount,
            "Wrong amount paid, must pay equal to the selling price!"
        );
        require!(
            current_time >= auction.start_time,
            "Cannot buy before start time!"
        );
        if auction.deadline != 0 {
            require!(
                current_time <= auction.deadline,
                "Cannot buy after deadline!"
            );
        }

        auction.current_winner = caller;
        auction.current_bid = payment_amount;
        auction.payment_token_nonce = payment_token_nonce;
        self.distribute_tokens(&auction, Some(&buy_amount));
        auction.nr_auctioned_tokens -= &buy_amount;
        if auction.nr_auctioned_tokens == 0 {
            self.listings_by_wallet(auction.original_owner.clone())
                .remove(&auction_id);
            self.token_auction_ids(nft_type.clone(), nft_nonce.clone())
                .remove(&auction_id);
            self.auction_by_id(auction_id).clear();
            self.listings().remove(&auction_id);
        } else {
            self.auction_by_id(auction_id).set(&auction);
        }

        let current_time = self.blockchain().get_block_timestamp();
        self.emit_buy_event(auction_id, auction, buy_amount, current_time);
    }

    #[payable("*")]
    #[endpoint(bulkBuy)]
    fn bulk_buy(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        #[var_args] auction_ids: MultiValueEncoded<u64>,
    ) {
        let mut total_available = payment_amount.clone();
        for auction_id in auction_ids.into_iter() {
            require!(
                self.does_auction_exist(auction_id),
                "One of your listings does not exist!"
            );
            let listing = self.try_get_auction(auction_id);
            require!(
                listing.auction_type == AuctionType::Nft,
                "You can bulk buy just NFTs on sell with a fixed price!"
            );
            require!(
                total_available >= listing.min_bid,
                "You do not have funds to buy all the NFTs!"
            );
            let buy_amount = listing.min_bid.clone();
            total_available -= listing.min_bid;
            self.buy(
                payment_token.clone(),
                payment_token_nonce,
                buy_amount,
                auction_id,
                listing.auctioned_token_type,
                listing.auctioned_token_nonce,
                OptionalValue::None,
            )
        }
        if total_available > BigUint::zero() {
            self.send().direct(
                &self.blockchain().get_caller(),
                &payment_token,
                payment_token_nonce,
                &total_available,
                &[],
            )
        }
    }

    #[payable("*")]
    #[endpoint(acceptOffer)]
    fn accept_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        offer_id: u64,
    ) {
        require!(self.status().get(), "Global operation enabled!");
        let mut offer = self.try_get_offer(offer_id);
        let current_time = self.blockchain().get_block_timestamp();
        require!(
            current_time <= offer.deadline,
            "Cannot accept the offer after deadline!"
        );
        let seller = self.blockchain().get_caller();
        require!(offer.offer_owner != seller, "Cannot accept your own offer!");
        let token_auction_ids_instance =
            self.token_auction_ids(offer.token_type.clone(), offer.token_nonce.clone());
        let mut found_match = false;
        let mut auction_removed = 0;
        if token_auction_ids_instance.is_empty() || payment_token.is_esdt() {
            require!(
                payment_amount == offer.quantity,
                "The quantity sent is not matching the offer!"
            );
            require!(
                payment_token_nonce == offer.token_nonce,
                "The nonce used is not matching the offer!"
            );
            require!(
                payment_token == offer.token_type,
                "The token sent is not matching the offer!"
            );
            found_match = true;
        } else if token_auction_ids_instance.len() == 1 {
            require!(
                token_auction_ids_instance.len() == 1,
                "You cannot accept offers for SFTs with more than 1 supply listed!"
            );
            let mut iter = token_auction_ids_instance.iter();
            let auction_id = iter.next().unwrap();
            let auction = self.try_get_auction(auction_id);
            require!(
                auction.auction_type == AuctionType::Nft,
                "Cannot accept offers for auctions, just for listings with a fixed price!"
            );

            require!(
                offer.offer_owner != auction.original_owner,
                "Cannot accept your own offer!"
            );

            require!(
                seller == auction.original_owner,
                "Just the owner of the listed NFT can accept the offer!"
            );

            require!(
                auction.nr_auctioned_tokens == offer.quantity,
                "The quantity listed is not matching the offer!"
            );
            require!(
                auction.auctioned_token_nonce == offer.token_nonce,
                "The nonce used is not matching the offer!"
            );
            require!(
                auction.auctioned_token_type == offer.token_type,
                "The listed token is not matching the offer!"
            );

            auction_removed = auction_id;
            self.listings_by_wallet(auction.original_owner.clone())
                .remove(&auction_id);
            self.token_auction_ids(offer.token_type.clone(), offer.token_nonce)
                .remove(&auction_id);
            self.auction_by_id(auction_id).clear();
            self.listings().remove(&auction_id);
            self.token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                .update(|qt| *qt -= &offer.quantity);

            if self
                .token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                .get()
                == BigUint::from(0u32)
            {
                self.token_items_for_sale(offer.token_type.clone())
                    .remove(&offer.token_nonce);
                self.token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                    .clear();
            }
            if self.token_items_for_sale(offer.token_type.clone()).len() == 0 {
                self.collections_listed().remove(&offer.token_type.clone());
            }

            found_match = true;
        } else {
            for auction_id in token_auction_ids_instance.iter() {
                let (
                    auctioned_token_type,
                    auctioned_token_nonce,
                    nr_auctioned_tokens,
                    owner_auction,
                    auction_type,
                ) = match self.get_auctioned_token_and_owner(auction_id) {
                    OptionalValue::Some(arg) => arg.into_tuple(),
                    OptionalValue::None => {
                        elrond_wasm::sc_panic!("The auction should have values!")
                    }
                };
                if offer.token_type == auctioned_token_type
                    && offer.token_nonce == auctioned_token_nonce
                    && offer.quantity == nr_auctioned_tokens
                    && seller == owner_auction
                    && (auction_type == AuctionType::Nft || auction_type == AuctionType::SftAll)
                {
                    auction_removed = auction_id;
                    self.listings_by_wallet(owner_auction).remove(&auction_id);
                    self.token_auction_ids(offer.token_type.clone(), offer.token_nonce)
                        .remove(&auction_id);
                    self.auction_by_id(auction_id).clear();
                    self.listings().remove(&auction_id);
                    self.token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                        .update(|qt| *qt -= &offer.quantity);

                    if self
                        .token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                        .get()
                        == BigUint::from(0u32)
                    {
                        self.token_items_for_sale(offer.token_type.clone())
                            .remove(&offer.token_nonce);
                        self.token_items_quantity_for_sale(
                            offer.token_type.clone(),
                            offer.token_nonce,
                        )
                        .clear();
                    }
                    found_match = true;
                    break;
                }
            }
        }

        require!(found_match, "No offer found for your accept!");
        offer.status = OfferStatus::Accepted;
        let nft_info = self.get_nft_info(&offer.token_type, offer.token_nonce);
        let creator_royalties_percentage = nft_info.royalties;
        require!(
            &offer.marketplace_cut_percentage + &creator_royalties_percentage < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );
        if !self.reward_ticker().is_empty() {
            if self
                .special_reward_amount(offer.token_type.clone())
                .is_empty()
            {
                if self.reward_balance().get().gt(&BigUint::from(0u32))
                    && self
                        .reward_balance()
                        .get()
                        .ge(&self.reward_amount().get().mul(2u32))
                {
                    self.transfer_or_save_payment(
                        &offer.offer_owner,
                        &self.reward_ticker().get(),
                        0u64,
                        &self.reward_amount().get(),
                        b"Trust Market rewards!",
                    );

                    self.transfer_or_save_payment(
                        &seller,
                        &self.reward_ticker().get(),
                        0u64,
                        &self.reward_amount().get(),
                        b"Trust Market rewards!",
                    );

                    self.reward_balance()
                        .update(|qt| *qt -= self.reward_amount().get().mul(2u32));
                }
            } else {
                if self.reward_balance().get().gt(&BigUint::from(0u32))
                    && self.reward_balance().get().ge(&self
                        .special_reward_amount(offer.token_type.clone())
                        .get()
                        .mul(2u32))
                {
                    self.transfer_or_save_payment(
                        &offer.offer_owner,
                        &self.reward_ticker().get(),
                        0u64,
                        &self.special_reward_amount(offer.token_type.clone()).get(),
                        b"Trust Market rewards!",
                    );

                    self.transfer_or_save_payment(
                        &seller,
                        &self.reward_ticker().get(),
                        0u64,
                        &self.special_reward_amount(offer.token_type.clone()).get(),
                        b"Trust Market rewards!",
                    );

                    self.reward_balance().update(|qt| {
                        *qt -= self
                            .special_reward_amount(offer.token_type.clone())
                            .get()
                            .mul(2u32)
                    });
                }
            }
        }
        self.transfer_or_save_payment(
            &offer.offer_owner,
            &offer.token_type,
            offer.token_nonce,
            &offer.quantity,
            b"Trust Market sent the bought token!",
        );

        let bid_split_amounts =
            self.calculate_offer_bid_split(&offer, &creator_royalties_percentage);

        let owner = self.blockchain().get_owner_address();
        self.transfer_or_save_payment(
            &owner,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &bid_split_amounts.marketplace,
            b"Trust Market fees revenue!",
        );

        self.transfer_or_save_payment(
            &nft_info.creator,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &bid_split_amounts.creator,
            b"Trust Market royalties for your token!",
        );

        // send rest of the offer to original seller
        self.transfer_or_save_payment(
            &seller,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &bid_split_amounts.seller,
            b"Trust Market income!",
        );
        self.check_offer_sent(
            offer.offer_owner.clone(),
            offer.token_type.clone(),
            offer.token_nonce.clone(),
            offer.payment_token_type.clone(),
        )
        .clear();
        self.token_offers_ids(offer.token_type.clone(), offer.token_nonce.clone())
            .remove(&offer_id);
        self.offers_by_wallet(offer.offer_owner.clone())
            .remove(&offer_id);
        self.offer_by_id(offer_id).clear();
        self.offers().remove(&offer_id);

        self.emit_accept_offer_event(offer_id, offer, &seller, auction_removed);
    }

    #[payable("*")]
    #[endpoint(declineOffer)]
    fn decline_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        offer_id: u64,
    ) {
        require!(self.status().get(), "Global operation enabled!");
        let offer = self.try_get_offer(offer_id);
        let owner = self.blockchain().get_caller();

        let token_auction_ids_instance =
            self.token_auction_ids(offer.token_type.clone(), offer.token_nonce.clone());
        if token_auction_ids_instance.is_empty() {
            require!(
                payment_amount == offer.quantity,
                "The quantity sent is not matching the offer!"
            );
            require!(
                payment_token_nonce == offer.token_nonce,
                "The nonce used is not matching the offer!"
            );
            require!(
                payment_token == offer.token_type,
                "The token sent is not matching the offer!"
            );
            self.send().direct(
                &owner,
                &payment_token,
                payment_token_nonce,
                &payment_amount,
                &[],
            );
        } else {
            require!(
                token_auction_ids_instance.len() == 1,
                "You cannot decline offers for SFTs with more than 1 supply minted!"
            );
            require!(
                payment_token_nonce == 0
                    && payment_token.is_egld()
                    && payment_amount.eq(&BigUint::zero()),
                "You have to send 0 eGLD as payment to decline the offer!"
            );
            let mut iter = token_auction_ids_instance.iter();
            let auction_id = iter.next().unwrap();
            let auction = self.try_get_auction(auction_id);
            require!(
                auction.auction_type == AuctionType::Nft,
                "Cannot decline offers for auctions, just for listings with a fixed price!"
            );
            require!(
                owner == auction.original_owner,
                "Just the owner of the NFT can decline the offer!"
            );
        }
        self.internal_withdraw_offer(offer_id);
    }

    #[endpoint]
    fn withdraw(&self, auction_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();

        require!(
            auction.original_owner == caller,
            "Only the original owner can withdraw!"
        );
        require!(
            auction.current_winner.is_zero()
                || auction.auction_type == AuctionType::SftOnePerPayment
                || auction.auction_type == AuctionType::Nft,
            "Cannot withdraw, the auction already has bids!"
        );

        self.distribute_tokens(&auction, Option::Some(&auction.nr_auctioned_tokens));

        self.token_auction_ids(
            auction.auctioned_token_type.clone(),
            auction.auctioned_token_nonce.clone(),
        )
        .remove(&auction_id);
        self.listings_by_wallet(auction.original_owner.clone())
            .remove(&auction_id);
        self.listings().remove(&auction_id);
        self.auction_by_id(auction_id).clear();
        self.emit_withdraw_event(auction_id, auction);
    }

    #[endpoint(withdrawOffer)]
    fn withdraw_offer(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let mut offer = self.try_get_offer(offer_id);
        let caller = self.blockchain().get_caller();

        require!(
            offer.offer_owner == caller,
            "Only the original owner can withdraw the offer!"
        );

        self.send().direct(
            &caller,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &offer.price,
            self.data_or_empty_if_sc(&caller, b"Trust Market withdraw offer!"),
        );

        self.token_offers_ids(offer.token_type.clone(), offer.token_nonce.clone())
            .remove(&offer_id);
        self.check_offer_sent(
            offer.offer_owner.clone(),
            offer.token_type.clone(),
            offer.token_nonce.clone(),
            offer.payment_token_type.clone(),
        )
        .clear();
        self.offers_by_wallet(offer.offer_owner.clone())
            .remove(&offer_id);
        self.offers().remove(&offer_id);
        self.offer_by_id(offer_id).clear();
        offer.status = OfferStatus::Withdraw;
        self.emit_withdraw_offer_event(offer_id, offer);
    }

    #[inline]
    fn internal_withdraw_offer(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let mut offer = self.try_get_offer(offer_id);

        self.send().direct(
            &offer.offer_owner,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &offer.price,
            self.data_or_empty_if_sc(&offer.offer_owner, b"Trust Market withdraw offer!"),
        );

        self.token_offers_ids(offer.token_type.clone(), offer.token_nonce.clone())
            .remove(&offer_id);
        self.check_offer_sent(
            offer.offer_owner.clone(),
            offer.token_type.clone(),
            offer.token_nonce.clone(),
            offer.payment_token_type.clone(),
        )
        .clear();
        self.offers_by_wallet(offer.offer_owner.clone())
            .remove(&offer_id);
        self.offers().remove(&offer_id);
        self.offer_by_id(offer_id).clear();
        offer.status = OfferStatus::Withdraw;
        self.emit_withdraw_offer_event(offer_id, offer);
    }

    #[endpoint(changePrice)]
    fn change_price(&self, auction_id: u64, new_price: BigUint) {
        require!(
            self.does_auction_exist(auction_id),
            "Auction does not exist!"
        );
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
        self.emit_change_price_event(auction_id, &auction, new_price.clone(), current_time);
        auction.max_bid = Some(new_price.clone());
        auction.min_bid = new_price.clone();
        self.auction_by_id(auction_id).set(auction);
    }

    fn try_get_auction(&self, auction_id: u64) -> Auction<Self::Api> {
        require!(
            self.does_auction_exist(auction_id),
            "Auction does not exist!"
        );
        self.auction_by_id(auction_id).get()
    }

    fn try_get_offer(&self, offer_id: u64) -> Offer<Self::Api> {
        require!(self.does_offer_exist(offer_id), "Offer does not exist!");
        self.offer_by_id(offer_id).get()
    }

    fn calculate_cut_amount(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / PERCENTAGE_TOTAL
    }

    fn calculate_winning_bid_split(
        &self,
        auction: &Auction<Self::Api>,
    ) -> BidSplitAmounts<Self::Api> {
        let creator_royalties =
            self.calculate_cut_amount(&auction.current_bid, &auction.creator_royalties_percentage);
        let bid_cut_amount =
            self.calculate_cut_amount(&auction.current_bid, &auction.marketplace_cut_percentage);
        let mut seller_amount_to_send = auction.current_bid.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &bid_cut_amount;

        BidSplitAmounts {
            creator: creator_royalties,
            marketplace: bid_cut_amount,
            seller: seller_amount_to_send,
        }
    }

    fn calculate_offer_bid_split(
        &self,
        offer: &Offer<Self::Api>,
        creator_royalties_percentage: &BigUint,
    ) -> BidSplitAmounts<Self::Api> {
        let creator_royalties =
            self.calculate_cut_amount(&offer.price, &creator_royalties_percentage);
        let bid_cut_amount =
            self.calculate_cut_amount(&offer.price, &offer.marketplace_cut_percentage);
        let mut seller_amount_to_send = offer.price.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &bid_cut_amount;

        BidSplitAmounts {
            creator: creator_royalties,
            marketplace: bid_cut_amount,
            seller: seller_amount_to_send,
        }
    }

    fn distribute_tokens(&self, auction: &Auction<Self::Api>, opt_sft_amount: Option<&BigUint>) {
        let nft_type = &auction.auctioned_token_type;
        let nft_nonce = auction.auctioned_token_nonce;
        if !auction.current_winner.is_zero() {
            let nft_info = self.get_nft_info(nft_type, nft_nonce);
            let token_id = &auction.payment_token_type;
            let nonce = auction.payment_token_nonce;
            let bid_split_amounts = self.calculate_winning_bid_split(auction);

            // send part as cut for contract owner
            let owner = self.blockchain().get_owner_address();
            self.transfer_or_save_payment(
                &owner,
                token_id,
                nonce,
                &bid_split_amounts.marketplace,
                b"Trust Market fees revenue!",
            );

            self.transfer_or_save_payment(
                &nft_info.creator,
                token_id,
                nonce,
                &bid_split_amounts.creator,
                b"Trust Market royalties for your token!",
            );

            // send rest of the bid to original owner
            self.transfer_or_save_payment(
                &auction.original_owner,
                token_id,
                nonce,
                &bid_split_amounts.seller,
                b"Trust Market income!",
            );
            if !self.reward_ticker().is_empty() {
                if self.special_reward_amount(nft_type.clone()).is_empty() {
                    if self.reward_balance().get().gt(&BigUint::from(0u32))
                        && self
                            .reward_balance()
                            .get()
                            .ge(&self.reward_amount().get().mul(2u32))
                    {
                        self.transfer_or_save_payment(
                            &auction.original_owner,
                            &self.reward_ticker().get(),
                            0u64,
                            &self.reward_amount().get(),
                            b"Trust Market rewards!",
                        );

                        self.transfer_or_save_payment(
                            &auction.current_winner,
                            &self.reward_ticker().get(),
                            0u64,
                            &self.reward_amount().get(),
                            b"Trust Market rewards!",
                        );
                        self.reward_balance()
                            .update(|qt| *qt -= self.reward_amount().get().mul(2u32));
                    }
                } else {
                    if self.reward_balance().get().gt(&BigUint::from(0u32))
                        && self
                            .reward_balance()
                            .get()
                            .ge(&self.special_reward_amount(nft_type.clone()).get().mul(2u32))
                    {
                        self.transfer_or_save_payment(
                            &auction.original_owner,
                            &self.reward_ticker().get(),
                            0u64,
                            &self.special_reward_amount(nft_type.clone()).get(),
                            b"Trust Market rewards!",
                        );

                        self.transfer_or_save_payment(
                            &auction.current_winner,
                            &self.reward_ticker().get(),
                            0u64,
                            &self.special_reward_amount(nft_type.clone()).get(),
                            b"Trust Market rewards!",
                        );

                        self.reward_balance().update(|qt| {
                            *qt -= self.special_reward_amount(nft_type.clone()).get().mul(2u32)
                        });
                    }
                }
            }
            // send NFT to auction winner
            let nft_amount = BigUint::from(NFT_AMOUNT);
            let nft_amount_to_send = match auction.auction_type {
                AuctionType::Nft => &nft_amount,
                AuctionType::NftBid => &nft_amount,
                AuctionType::SftOnePerPayment => match opt_sft_amount {
                    Some(amt) => amt,
                    None => &nft_amount,
                },
                _ => &auction.nr_auctioned_tokens,
            };
            self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .update(|qt| *qt -= nft_amount_to_send.clone());

            if self
                .token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .get()
                .eq(&BigUint::from(0u32))
            {
                self.token_items_for_sale(nft_type.clone())
                    .remove(&nft_nonce);
                self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                    .clear();
            }
            if self.token_items_for_sale(nft_type.clone()).len() == 0 {
                self.collections_listed().remove(&nft_type);
            }

            self.transfer_or_save_payment(
                &auction.current_winner,
                nft_type,
                nft_nonce,
                nft_amount_to_send,
                b"Trust Market sent the bought token!",
            );
        } else {
            // return to original owner

            self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .update(|qt| *qt -= &auction.nr_auctioned_tokens);
            let quantity_token = self
                .token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .get();
            if quantity_token.eq(&BigUint::from(0u32)) {
                self.token_items_for_sale(nft_type.clone())
                    .remove(&nft_nonce);
                self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                    .clear();
            }

            if self.token_items_for_sale(nft_type.clone()).len() == 0 {
                self.collections_listed().remove(&nft_type);
            }

            self.transfer_or_save_payment(
                &auction.original_owner,
                nft_type,
                nft_nonce,
                &auction.nr_auctioned_tokens,
                b"Trust Market returned your token!",
            );
        }
    }

    #[endpoint(claimTokens)]
    fn claim_tokens(
        &self,
        token_id: TokenIdentifier,
        token_nonce: u64,
        claim_destination: ManagedAddress,
    ) {
        let caller = self.blockchain().get_caller();
        let amount_mapper = self.claimable_amount(&caller, &token_id, token_nonce);
        let amount = amount_mapper.get();

        if amount > 0 {
            amount_mapper.clear();

            self.send()
                .direct(&claim_destination, &token_id, token_nonce, &amount, &[]);
        }
    }

    fn transfer_or_save_payment(
        &self,
        to: &ManagedAddress,
        token_id: &TokenIdentifier,
        nonce: u64,
        amount: &BigUint,
        data: &'static [u8],
    ) {
        if self.blockchain().is_smart_contract(to) && !self.whitelisted_contracts().contains(&to) {
            self.claimable_tokens(to).insert(token_id.clone());
            self.claimable_token_nonces(to, token_id).insert(nonce);
            self.claimable_amount(to, token_id, nonce)
                .update(|amt| *amt += amount);
        } else {
            self.send().direct(
                to,
                token_id,
                nonce,
                amount,
                self.data_or_empty_if_sc(to, data),
            );
        }
    }

    fn data_or_empty_if_sc(&self, dest: &ManagedAddress, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(dest) {
            &[]
        } else {
            data
        }
    }

    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }

    fn try_set_bid_cut_percentage(&self, new_cut_percentage: u64) {
        require!(
            new_cut_percentage > 0 && new_cut_percentage < PERCENTAGE_TOTAL,
            "Invalid percentage value, should be between 0 and 10,000"
        );

        self.bid_cut_percentage()
            .set(&BigUint::from(new_cut_percentage));
    }

    #[endpoint(cleanExpiredOffers)]
    fn clean_expired_offers(&self) -> i32 {
        let timestamp = self.blockchain().get_block_timestamp();
        let mut found = 0;
        for offer_id in self.offers().iter() {
            let offer = self.offer_by_id(offer_id);
            if !offer.is_empty() {
                if offer.get().deadline < timestamp {
                    found += 1;
                    self.internal_withdraw_offer(offer_id);
                }
                if found == 150 {
                    break;
                }
            } else {
                self.offers().remove(&offer_id);
            }
        }
        found
    }
}
