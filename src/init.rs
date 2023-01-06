#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();
pub mod auction;
use auction::*;
pub mod admin;
pub mod common;
pub mod creator;
pub mod dex;
pub mod events;
pub mod helpers;
pub mod offers;
pub mod storage;
pub mod views;
pub mod wrapping;

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
const NFT_AMOUNT: u32 = 1; // Token has to be unique to be considered NFT

#[elrond_wasm::contract]
pub trait XOXNOProtocol:
    storage::StorageModule
    + views::ViewsModule
    + events::EventsModule
    + helpers::HelpersModule
    + offers::CustomOffersModule
    + admin::AdminModule
    + creator::CreatorModule
    + common::CommonModule
    + wrapping::WrappingModule
    + dex::DexModule
{
    #[init]
    fn init(
        &self,
        bid_cut_percentage: u64,
        signer: ManagedAddress,
        wrapping_sc: ManagedAddress,
        wrapping_token: TokenIdentifier,
        xoxno_pair: ManagedAddress,
        xoxno_token: TokenIdentifier,
    ) {
        self.try_set_bid_cut_percentage(bid_cut_percentage);
        self.signer().set_if_empty(&signer);
        self.wrapping().set(wrapping_sc);
        self.wrapping_token().set(wrapping_token);
        self.swap_pair_xoxno().set(xoxno_pair);
        self.xoxno_token().set(xoxno_token);
    }

    #[payable("*")]
    #[endpoint(listing)]
    fn listing(&self, listings: MultiValueEncoded<BulkListing<Self::Api>>) {
        require!(self.status().get(), "Global operation enabled!");
        let payments = self.call_value().all_esdt_transfers();
        let marketplace_cut_percentage = &self.bid_cut_percentage().get();
        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();

        let mut map_listings = self.listings();
        let mut map_caller_listings = self.listings_by_wallet(&caller);
        let mut map_collections = self.collections_listed();
        let map_acc_tokens = self.accepted_tokens();

        require!(listings.len() == payments.len(), "Invalid body sent!");
        for (index, listing) in listings.to_vec().iter().enumerate() {
            let (nft_type, nft_nonce, nft_amount) = payments.get(index).into_tuple();
            require!(
                map_acc_tokens.contains(&listing.accepted_payment_token),
                "The payment token is not whitelisted!"
            );
            require!(
                nft_amount >= BigUint::from(NFT_AMOUNT),
                "Must tranfer at least one"
            );
            require!(
                nft_nonce == listing.nonce
                    && nft_type == listing.collection
                    && nft_amount == listing.nft_amount,
                "The payment item is not matching the listing item"
            );
            let start_time = if listing.opt_start_time == 0 {
                current_time
            } else {
                listing.opt_start_time
            };

            let sft_max_one_per_payment = listing.opt_sft_max_one_per_payment;

            if sft_max_one_per_payment || !listing.bid {
                require!(
                    listing.min_bid == listing.max_bid,
                    "Price must be fixed for this type of auction (min bid equal to max bid)"
                );
            }
            if !listing.accepted_payment_token.is_egld() {
                require!(
                    listing.accepted_payment_token.is_esdt(),
                    "The payment token is not valid!"
                );
            }

            let opt_max_bid = if listing.max_bid > 0u32 {
                require!(
                    listing.min_bid <= listing.max_bid,
                    "Min bid can't higher than max bid"
                );

                Some(&listing.max_bid)
            } else {
                None
            };

            require!(listing.min_bid > 0u32, "Min bid must be higher than 0!");
            require!(
                nft_nonce > 0,
                "Only Semi-Fungible and Non-Fungible tokens can be auctioned"
            );
            require!(
                listing.deadline > current_time || listing.deadline == 0,
                "Deadline can't be in the past"
            );
            if listing.deadline != 0 {
                require!(
                    start_time >= current_time && start_time < listing.deadline,
                    "Invalid start time"
                );
            }

            let creator_royalties_percentage = self.get_nft_info(&nft_type, nft_nonce).royalties;

            require!(
                marketplace_cut_percentage + &creator_royalties_percentage < PERCENTAGE_TOTAL,
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
                match listing.bid {
                    true => AuctionType::NftBid,
                    false => AuctionType::Nft,
                }
            };

            if listing.deadline == 0 {
                require!(
                    auction_type == AuctionType::Nft
                        || auction_type == AuctionType::SftOnePerPayment
                        || (auction_type == AuctionType::SftAll
                            && &listing.min_bid == &listing.max_bid),
                    "Deadline is mandatory for this auction type!"
                );
            }

            let auction = Auction {
                auctioned_token_type: nft_type.clone(),
                auctioned_token_nonce: nft_nonce,

                nr_auctioned_tokens: nft_amount.clone(),
                auction_type,

                payment_token_type: listing.accepted_payment_token,
                payment_token_nonce: accepted_payment_nft_nonce,

                min_bid: listing.min_bid,
                max_bid: opt_max_bid.cloned(),
                start_time,
                deadline: listing.deadline,
                original_owner: caller.clone(),
                current_bid: BigUint::zero(),
                current_winner: ManagedAddress::zero(),
                marketplace_cut_percentage: marketplace_cut_percentage.clone(),
                creator_royalties_percentage,
            };

            // Map ID with Auction Struct
            self.auction_by_id(auction_id).set(&auction);
            map_listings.insert(auction_id); // Push ID to the auctions list

            // Add to the owner wallet the new Auction ID
            map_caller_listings.insert(auction_id);
            // Insert nonce for sale per collection
            self.token_items_for_sale(&nft_type).insert(nft_nonce);
            // Insert auction ID per token and nonce
            self.token_auction_ids(&nft_type, nft_nonce)
                .insert(auction_id);

            self.token_items_quantity_for_sale(&nft_type, nft_nonce)
                .update(|qt| *qt += &nft_amount);

            map_collections.insert(nft_type);
            //Emit event for new listed token
            self.emit_auction_token_event(auction_id, auction);
        }
    }

    #[payable("*")]
    #[endpoint]
    fn bid(&self, auction_id: u64, nft_type: TokenIdentifier, nft_nonce: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let (payment_token, payment_token_nonce, payment_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();

        let mut auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();
        let wegld = self.wrapping_token().get();
        self.common_bid_checks(
            &auction,
            &nft_type,
            nft_nonce,
            &payment_token,
            payment_token_nonce,
            &payment_amount,
            &wegld,
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
            self.emit_out_bid_event(auction_id, &auction, &caller, &payment_amount);
        }
        let wrapping = self.require_egld_conversion(&auction, &payment_token, &wegld);
        if wrapping {
            // update auction bid and winner
            auction.current_bid = payment_amount.clone();
            if auction.payment_token_type.is_egld() {
                self.unwrap_egld(payment_amount);
            } else if auction.payment_token_type.is_esdt() {
                self.wrap_egld(payment_amount);
            }
        } else {
            // update auction bid and winner
            auction.current_bid = payment_amount;
        }
        auction.current_winner = caller;
        self.auction_by_id(auction_id).set(&auction);
        self.listings_bids(&auction.current_winner)
            .insert(auction_id);

        if max_bid_reached {
            self.end_auction_common(auction_id, &auction);
        } else {
            self.emit_bid_event(auction_id, auction);
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
        self.end_auction_common(auction_id, &auction);
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
        let mut bought_nfts: ManagedVec<EsdtTokenPayment<Self::Api>> = ManagedVec::new();
        let caller = self.blockchain().get_caller();
        let wegld = self.wrapping_token().get();
        let mut marketplace_fees = BigUint::zero();
        for auction_id in auction_ids.into_iter() {
            let listing_map = self.auction_by_id(auction_id);
            if listing_map.is_empty() {
                continue;
            }
            let mut listing = listing_map.get();
            require!(
                listing.auction_type == AuctionType::Nft,
                "You can bulk buy just NFTs on sell with a fixed price!"
            );

            require!(
                total_available >= listing.min_bid,
                "You do not have funds to buy all the NFTs!"
            );

            self.common_bid_checks(
                &listing,
                &listing.auctioned_token_type,
                listing.auctioned_token_nonce,
                &payment_token,
                payment_token_nonce,
                &total_available,
                &wegld,
            );

            let wrapping = self.require_egld_conversion(&listing, &payment_token, &wegld);
            let nft_info =
                self.get_nft_info(&listing.auctioned_token_type, listing.auctioned_token_nonce);

            listing.current_bid = listing.min_bid.clone();
            let bid_split_amounts = self.calculate_winning_bid_split(&listing);

            self.distribute_tokens_bulk_buy(
                &listing.payment_token_type,
                listing.payment_token_nonce,
                &nft_info.creator,
                &listing.original_owner,
                &bid_split_amounts,
                wrapping,
            );

            marketplace_fees += bid_split_amounts.marketplace;
            self.update_or_remove_items_quantity(&listing, &listing.nr_auctioned_tokens);
            self.remove_auction_common(auction_id, &listing);
            total_available -= listing.min_bid;
            bought_nfts.push(EsdtTokenPayment::new(
                listing.auctioned_token_type,
                listing.auctioned_token_nonce,
                listing.nr_auctioned_tokens,
            ));
        }
        if total_available > BigUint::zero() {
            self.send().direct(
                &caller,
                &payment_token,
                payment_token_nonce,
                &total_available,
            )
        }
        if bought_nfts.len() > 0 {
            self.send().direct_multi(&caller, &bought_nfts)
        }
        if marketplace_fees > BigUint::zero() {
            self.share_marketplace_fees(
                &payment_token,
                marketplace_fees,
                payment_token_nonce,
                wegld,
                false,
            );
        }

    }

    #[endpoint]
    fn withdraw(&self, withdraws: MultiValueEncoded<u64>) {
        require!(self.status().get(), "Global operation enabled!");
        let caller = self.blockchain().get_caller();
        for auction_id in withdraws.into_iter() {
            let listing_map = self.auction_by_id(auction_id);
            if listing_map.is_empty() {
                continue;
            }
            let listing = listing_map.get();
            require!(
                &listing.original_owner == &caller,
                "Only the original owner can withdraw!"
            );
            self.withdraw_auction_common(auction_id, &listing);
        }
    }

    #[endpoint(changeListing)]
    fn bulk_change_listing(&self, updates: MultiValueEncoded<BulkUpdateListing<Self::Api>>) {
        require!(self.status().get(), "Global operation enabled!");
        let caller = self.blockchain().get_caller();
        require!(updates.len() > 0, "You can not send len 0 of updates!");
        for update in updates.into_iter() {
            let listing_map = self.auction_by_id(update.auction_id);
            if listing_map.is_empty() {
                continue;
            }
            let mut listing = listing_map.get();

            require!(
                listing.auction_type == AuctionType::Nft
                    || listing.auction_type == AuctionType::SftOnePerPayment,
                "Only NFT and SftOnePerPayment auctions can be bulk updated",
            );
            require!(
                listing.original_owner == caller,
                "Only the original owner can change the listing info!"
            );
            require!(
                listing.auction_type == AuctionType::Nft
                    || listing.auction_type == AuctionType::SftOnePerPayment,
                "You can not change the price of bids!"
            );
            self.emit_change_listing_event(update.auction_id, &listing, &update.new_price);
            listing.min_bid = update.new_price.clone();
            listing.max_bid = Some(update.new_price.clone());
            listing.payment_token_type = update.payment_token_type;
            listing.deadline = update.deadline;
            listing_map.set(listing);
        }
    }
}
