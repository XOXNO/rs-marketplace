#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();
pub mod auction;
use crate::aggregator::{AggregatorStep, TokenAmount};
use auction::*;
pub mod accumulator;
pub mod admin;
pub mod aggregator;
pub mod common;
pub mod creator;
pub mod events;
pub mod helpers;
pub mod offers;
pub mod pools;
pub mod storage;
pub mod views;
pub mod wrapping;

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
const MAX_COLLECTION_ROYALTIES: u64 = 5_000; // 50%
const NFT_AMOUNT: u32 = 1; // Token has to be unique to be considered NFT
const MIN_TRADE_REWARD: u64 = 200_000_000_000_000_000; // Token has to be unique to be considered NFT

#[multiversx_sc::contract]
pub trait XOXNOProtocol:
    storage::StorageModule
    + views::ViewsModule
    + events::EventsModule
    + helpers::HelpersModule
    + offers::CustomOffersModule
    + admin::AdminModule
    + creator::CreatorModule
    + wrapping::WrappingModule
    + common::CommonModule
    + pools::PoolsModule
{
    #[init]
    fn init(
        &self,
        bid_cut_percentage: u64,
        signer: ManagedAddress,
        wrapping_sc: ManagedAddress,
        wrapping_token: TokenIdentifier,
        aggregator: ManagedAddress,
        // xoxno_token: TokenIdentifier,
    ) {
        self.try_set_bid_cut_percentage(bid_cut_percentage);
        self.signer().set_if_empty(&signer);
        self.wrapping().set(wrapping_sc);
        self.wrapping_token().set(wrapping_token);
        self.aggregator_sc().set(aggregator);
        // self.xoxno_token().set(xoxno_token);
    }

    #[upgrade]
    fn upgrade(&self, sc_accumulator: ManagedAddress, aggregator: ManagedAddress) {
        self.accumulator().set(sc_accumulator);
        self.aggregator_sc().set(aggregator);
    }

    #[payable("*")]
    #[endpoint(listing)]
    fn listing(&self, listings: MultiValueEncoded<BulkListing<Self::Api>>) {
        self.require_enabled();
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
            let (nft_type, nft_nonce, nft_amount) = payments.get(index).clone().into_tuple();
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
            let fee_map = self.collection_config(&nft_type);
            let mut creator_royalties_percentage =
                self.get_nft_info(&nft_type, nft_nonce).royalties;

            if !fee_map.is_empty() {
                let fee_config = fee_map.get();
                if fee_config.custom_royalties {
                    creator_royalties_percentage = listing.royalties.clone();
                    if creator_royalties_percentage > fee_config.max_royalties {
                        creator_royalties_percentage = fee_config.max_royalties;
                    } else if creator_royalties_percentage < fee_config.min_royalties {
                        creator_royalties_percentage = fee_config.min_royalties;
                    }
                }
            }

            if marketplace_cut_percentage + &creator_royalties_percentage >= PERCENTAGE_TOTAL {
                creator_royalties_percentage = BigUint::from(MAX_COLLECTION_ROYALTIES);
            }

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

                payment_token_type: listing.accepted_payment_token.clone(),
                payment_token_nonce: accepted_payment_nft_nonce,

                min_bid: listing.min_bid.clone(),
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
    #[endpoint(bid)]
    fn bid(&self, auction_id: u64, nft_type: TokenIdentifier, nft_nonce: u64) {
        self.require_enabled();
        let (payment_token, payment_token_nonce, payment_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();
        require!(
            !self.freezed_auctions().contains(&auction_id),
            "Auction is frozen!"
        );
        let mut auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();
        let wegld = self.wrapping_token().get();
        self.common_bid_checks(
            &auction,
            auction_id,
            &nft_type,
            nft_nonce,
            &payment_token,
            payment_token_nonce,
            &payment_amount,
            &wegld,
            false,
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
        self.require_enabled();
        let auction = self.try_get_auction(auction_id);
        require!(
            !self.freezed_auctions().contains(&auction_id),
            "Auction is frozen!"
        );
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
            deadline_reached || max_bid_reached || auction.current_winner == ManagedAddress::zero(),
            "Auction deadline has not passed or the current bid is not equal to the max bid!"
        );

        if auction.current_winner == ManagedAddress::zero() {
            require!(
                self.blockchain().get_caller() == auction.original_owner,
                "You are not the owner of this auction in order to withdraw it!"
            );
        }

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
            OptionalValue::None,
            OptionalValue::None,
        );
    }

    #[allow_multiple_var_args]
    #[payable("*")]
    #[endpoint(buySwap)]
    fn buy_swap(
        &self,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        steps: ManagedVec<AggregatorStep<Self::Api>>,
        limits: ManagedVec<TokenAmount<Self::Api>>,
        opt_sft_buy_amount: OptionalValue<BigUint>,
    ) {
        self.common_buy(
            auction_id,
            nft_type,
            nft_nonce,
            opt_sft_buy_amount,
            OptionalValue::None,
            OptionalValue::None,
            OptionalValue::Some(steps),
            OptionalValue::Some(limits),
        );
    }

    #[allow_multiple_var_args]
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
            OptionalValue::None,
            OptionalValue::None,
        );
    }

    #[payable("*")]
    #[endpoint(bulkBuy)]
    fn bulk_buy(
        &self,
        auction_ids: MultiValueEncoded<u64>,
    ) -> ManagedVec<EsdtTokenPayment<Self::Api>> {
        let payments = self.call_value().egld_or_single_esdt();
        let mut total_available = payments.amount.clone();
        let mut bought_nfts: ManagedVec<EsdtTokenPayment<Self::Api>> = ManagedVec::new();
        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();
        let wegld = self.wrapping_token().get();
        let mut marketplace_fees = BigUint::zero();

        let map_frozen = self.freezed_auctions();

        for auction_id in auction_ids.into_iter() {
            let listing_map = self.auction_by_id(auction_id);
            if listing_map.is_empty() {
                continue;
            }
            require!(!map_frozen.contains(&auction_id), "Auction is frozen!");
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
                auction_id,
                &listing.auctioned_token_type,
                listing.auctioned_token_nonce,
                &payments.token_identifier,
                payments.token_nonce,
                &total_available,
                &wegld,
                false,
            );

            let wrapping =
                self.require_egld_conversion(&listing, &payments.token_identifier, &wegld);
            let nft_info =
                self.get_nft_info(&listing.auctioned_token_type, listing.auctioned_token_nonce);

            listing.current_bid = listing.min_bid.clone();
            listing.current_winner = caller.clone();
            let config = self.get_collection_config(&listing.auctioned_token_type);

            let bid_split_amounts = self.calculate_amount_split(
                &listing.current_bid,
                &listing.creator_royalties_percentage,
                config.clone(),
            );

            if config.is_some() {
                if config.unwrap().reverse_cut_fees {
                    total_available += &bid_split_amounts.marketplace;
                } else {
                    marketplace_fees += &bid_split_amounts.marketplace;
                }
            } else {
                marketplace_fees += &bid_split_amounts.marketplace;
            }

            self.distribute_tokens_bulk_buy(
                &listing.payment_token_type,
                listing.payment_token_nonce,
                &nft_info.creator,
                &listing.original_owner,
                &caller,
                &bid_split_amounts,
                wrapping,
            );
            self.update_or_remove_items_quantity(&listing, &listing.nr_auctioned_tokens);
            self.remove_auction_common(auction_id, &listing);
            self.emit_buy_event(
                auction_id,
                &listing,
                &listing.nr_auctioned_tokens,
                current_time,
                OptionalValue::None,
                OptionalValue::None,
                &payments,
            );
            total_available -= listing.min_bid;

            bought_nfts.push(EsdtTokenPayment::new(
                listing.auctioned_token_type,
                listing.auctioned_token_nonce,
                listing.nr_auctioned_tokens,
            ));
        }

        if total_available.gt(&BigUint::zero()) {
            self.send().direct(
                &caller,
                &payments.token_identifier,
                payments.token_nonce,
                &total_available,
            )
        }

        if bought_nfts.len() > 0 {
            self.send().direct_multi(&caller, &bought_nfts)
        }

        if marketplace_fees > BigUint::zero() {
            self.share_marketplace_fees(
                &payments.token_identifier,
                marketplace_fees,
                payments.token_nonce,
            );
        }
        bought_nfts
    }

    #[allow_multiple_var_args]
    #[endpoint(withdraw)]
    fn withdraw(&self, signature: ManagedBuffer, withdraws: MultiValueEncoded<u64>) {
        self.require_enabled();
        let caller = self.blockchain().get_caller();
        let map_frozen = self.freezed_auctions();
        let sign = signature;
        let has_sign = false;
        // require!(sign.is_some(), "Signature required!");
        let mut data = ManagedBuffer::new();
        if has_sign {
            data.append(caller.as_managed_buffer());
        }

        for auction_id in withdraws.into_iter() {
            if has_sign {
                data.append(&self.decimal_to_ascii(auction_id.try_into().unwrap()));
            }
            let listing_map = self.auction_by_id(auction_id);
            if listing_map.is_empty() {
                continue;
            }
            require!(!map_frozen.contains(&auction_id), "Auction is frozen!");
            let listing = listing_map.get();
            require!(
                &listing.original_owner == &caller,
                "Only the original owner can withdraw!"
            );
            self.withdraw_auction_common(auction_id, &listing);
        }
        if has_sign {
            let signer: ManagedAddress = self.signer().get();
            self.crypto()
                .verify_ed25519(signer.as_managed_buffer(), &data, &sign);
        }
    }

    #[endpoint(changeListing)]
    fn bulk_change_listing(&self, updates: MultiValueEncoded<BulkUpdateListing<Self::Api>>) {
        self.require_enabled();
        let caller = self.blockchain().get_caller();
        require!(updates.len() > 0, "You can not send len 0 of updates!");
        let map_frozen = self.freezed_auctions();
        for update in updates.into_iter() {
            let listing_map = self.auction_by_id(update.auction_id);
            if listing_map.is_empty() {
                // skip in case of already removed auctions to avoid failing the entire TX
                continue;
            }
            require!(
                !map_frozen.contains(&update.auction_id),
                "Auction is frozen!"
            );
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
            listing.payment_token_type = update.payment_token_type;
            listing.deadline = update.deadline;
            self.emit_change_listing_event(update.auction_id, &listing, &update.new_price);
            listing.min_bid = update.new_price.clone();
            listing.max_bid = Some(update.new_price.clone());
            listing_map.set(listing);
        }
    }
}
