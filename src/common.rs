multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{
    accumulator,
    auction::{Auction, AuctionType, FeesDistribution, GlobalOffer, Offer},
    MIN_TRADE_REWARD, NFT_AMOUNT,
};

#[multiversx_sc::module]
pub trait CommonModule:
    crate::storage::StorageModule
    + crate::helpers::HelpersModule
    + crate::views::ViewsModule
    + crate::events::EventsModule
    + crate::wrapping::WrappingModule
{
    fn withdraw_auction_common(&self, auction_id: u64, auction: &Auction<Self::Api>) {
        require!(
            auction.current_winner.is_zero()
                || auction.auction_type == AuctionType::SftOnePerPayment
                || auction.auction_type == AuctionType::Nft,
            "Cannot withdraw, the auction already has bids!"
        );

        self.update_or_remove_items_quantity(auction, &auction.nr_auctioned_tokens);
        self.remove_auction_common(auction_id, auction);
        self.return_auction_nft(auction);
        self.emit_withdraw_event(auction_id, auction);
    }

    fn end_auction_common(&self, auction_id: u64, auction: &Auction<Self::Api>) {
        self.update_or_remove_items_quantity(auction, &auction.nr_auctioned_tokens);
        self.remove_auction_common(auction_id, auction);
        self.emit_end_auction_event(auction_id, auction);
        self.distribute_tokens(auction, Option::Some(&auction.nr_auctioned_tokens), false);
    }

    fn common_bid_checks(
        &self,
        auction: &Auction<Self::Api>,
        auction_id: u64,
        nft_type: &TokenIdentifier,
        nft_nonce: u64,
        payment_token: &EgldOrEsdtTokenIdentifier,
        payment_nonce: u64,
        payment_amount: &BigUint,
        wegld: &TokenIdentifier,
        require_swap: bool,
    ) {
        let caller = self.blockchain().get_caller();
        let current_time = self
            .blockchain()
            .get_block_timestamp_seconds()
            .as_u64_seconds();
        require!(
            !self.freezed_auctions().contains(&auction_id),
            "Auction is frozen!"
        );
        require!(
            &auction.auctioned_token_type == nft_type && auction.auctioned_token_nonce == nft_nonce,
            "Auction ID does not match the token"
        );
        require!(
            auction.original_owner != caller,
            "Can't bid on your own token"
        );
        require!(
            current_time >= auction.start_time,
            "Auction hasn't started yet"
        );
        if auction.deadline != 0
            && !(auction.auction_type == AuctionType::SftAll
                || auction.auction_type == AuctionType::NftBid)
        {
            require!(current_time < auction.deadline, "Auction ended already");
        }

        if auction.auction_type == AuctionType::SftAll
            || auction.auction_type == AuctionType::NftBid
        {
            require!(
                current_time < auction.deadline
                    || (auction.deadline == 0
                        && AuctionType::SftAll == auction.auction_type
                        && auction.max_bid.is_some()
                        && auction.min_bid == auction.max_bid.clone().unwrap()),
                "Auction ended already!"
            );

            require!(auction.current_winner != caller, "Can't outbid yourself!");

            require!(
                payment_amount >= &auction.min_bid,
                "Bid must be higher than or equal to the min bid!"
            );
            require!(
                payment_amount > &auction.current_bid,
                "Bid must be higher than the current winning bid!"
            );
        }

        let is_egld_or_wegld = payment_token.is_egld() || payment_token == wegld;
        let valid_payment_egld_or_wegld = (is_egld_or_wegld
            && auction.payment_token_type.is_egld())
            || (auction.payment_token_type.is_esdt()
                && &auction.payment_token_type == wegld
                && is_egld_or_wegld);

        if !require_swap {
            require!(
                payment_token == &auction.payment_token_type
                    && payment_nonce == auction.payment_token_nonce
                    || valid_payment_egld_or_wegld,
                "Wrong token used as payment"
            );
        }
    }

    #[allow_multiple_var_args]
    fn common_buy(
        &self,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        opt_sft_buy_amount: OptionalValue<BigUint>,
        buy_for: OptionalValue<ManagedAddress>,
        message: OptionalValue<ManagedBuffer>,
        swaps: OptionalValue<ManagedArgBuffer<Self::Api>>,
    ) {
        self.require_enabled();
        let payments = self.call_value().egld_or_single_esdt();
        let (payment_token, payment_token_nonce, payment_amount) = payments.clone().into_tuple();
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

        let wegld = self.wrapping_token().get();
        let require_swap = swaps.is_some() && auction.payment_token_type != payment_token;
        self.common_bid_checks(
            &auction,
            auction_id,
            &nft_type,
            nft_nonce,
            &payment_token,
            payment_token_nonce,
            &payment_amount,
            &wegld,
            require_swap,
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

        if !require_swap {
            require!(
                total_value == payment_amount,
                "Wrong amount paid, must pay equal to the selling price!"
            );
            auction.current_winner = buyer.clone();
            auction.current_bid = payment_amount;
            auction.nr_auctioned_tokens -= &buy_amount;
            if auction.nr_auctioned_tokens == 0 {
                self.remove_auction_common(auction_id, &auction);
            } else {
                self.auction_by_id(auction_id).set(&auction);
            }
            self.update_or_remove_items_quantity(&auction, &buy_amount);

            let current_time = self
                .blockchain()
                .get_block_timestamp_seconds()
                .as_u64_seconds();
            self.emit_buy_event(
                auction_id,
                &auction,
                &buy_amount,
                current_time,
                message,
                if buy_for.into_option().is_some() {
                    OptionalValue::Some(caller)
                } else {
                    OptionalValue::None
                },
                &payments,
            );
            let wrapping = self.require_egld_conversion(&auction, &payment_token, &wegld);
            self.distribute_tokens(&auction, Option::Some(&buy_amount), wrapping);
        } else {
            let steps = swaps.into_option().unwrap();
            self.freezed_auctions().insert(auction_id);

            self.aggregate(
                buyer,
                &caller,
                &buy_amount,
                &total_value,
                auction_id,
                payments,
                steps,
                message,
            );
        }
    }

    fn distribute_tokens(
        &self,
        auction: &Auction<Self::Api>,
        opt_sft_amount: Option<&BigUint>,
        wrapping: bool,
    ) {
        if !auction.current_winner.is_zero() {
            let nft_info =
                self.get_nft_info(&auction.auctioned_token_type, auction.auctioned_token_nonce);

            let bid_split_amounts = self.calculate_amount_split(
                &auction.current_bid,
                &auction.creator_royalties_percentage,
                self.get_collection_config(&auction.auctioned_token_type),
            );

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

            self.distribute_tokens_common(
                ManagedVec::from(EsdtTokenPayment::new(
                    auction.auctioned_token_type.clone(),
                    auction.auctioned_token_nonce,
                    nft_amount_to_send.clone(),
                )),
                &auction.payment_token_type,
                auction.payment_token_nonce,
                &nft_info.creator,
                &auction.original_owner,
                &auction.current_winner,
                &bid_split_amounts,
                wrapping,
            );
        } else {
            self.return_auction_nft(auction);
        }
    }

    fn require_egld_conversion(
        &self,
        auction: &Auction<Self::Api>,
        payment: &EgldOrEsdtTokenIdentifier,
        wegld: &TokenIdentifier,
    ) -> bool {
        auction.payment_token_type.is_egld() && payment.is_esdt() && payment.eq(wegld)
            || auction.payment_token_type.eq(wegld) && payment.is_egld()
    }

    fn return_auction_nft(&self, auction: &Auction<Self::Api>) {
        self.transfer_or_save_payment(
            &auction.original_owner,
            &EgldOrEsdtTokenIdentifier::esdt(auction.auctioned_token_type.clone()),
            auction.auctioned_token_nonce,
            &auction.nr_auctioned_tokens,
        );
    }

    fn update_or_remove_items_quantity(&self, auction: &Auction<Self::Api>, quantity: &BigUint) {
        let quantity_token = self.token_items_quantity_for_sale(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
        );
        quantity_token.update(|qt| *qt -= quantity);
        let mut map_token_for_sale = self.token_items_for_sale(&auction.auctioned_token_type);
        if quantity_token.get().eq(&BigUint::zero()) {
            map_token_for_sale.remove(&auction.auctioned_token_nonce);
            quantity_token.clear();
        }

        if map_token_for_sale.is_empty() {
            self.collections_listed()
                .remove(&auction.auctioned_token_type);
        }
    }

    fn remove_auction_common(&self, auction_id: u64, auction: &Auction<Self::Api>) {
        self.token_auction_ids(&auction.auctioned_token_type, auction.auctioned_token_nonce)
            .remove(&auction_id);
        self.listings_by_wallet(&auction.original_owner)
            .remove(&auction_id);
        self.listings().remove(&auction_id);
        if !auction.current_winner.is_zero() {
            self.listings_bids(&auction.current_winner)
                .remove(&auction_id);
        }
        self.auction_by_id(auction_id).clear();
    }

    fn common_global_offer_remove(&self, offer: &GlobalOffer<Self::Api>, return_offer: bool) {
        if return_offer && !offer.new_version {
            self.transfer_or_save_payment(
                &offer.owner,
                &offer.payment_token,
                offer.payment_nonce,
                &offer.price,
            );
        }
        self.user_collection_global_offers(&offer.owner, &offer.collection)
            .swap_remove(&offer.offer_id);
        self.collection_global_offers(&offer.collection)
            .swap_remove(&offer.offer_id);
        self.user_global_offers(&offer.owner)
            .swap_remove(&offer.offer_id);
        self.global_offer(offer.offer_id).clear();
        self.global_offer_ids().swap_remove(&offer.offer_id);

        if return_offer {
            self.emit_remove_global_offer_event(offer.offer_id, &offer.collection);
        }
    }

    fn common_withdraw_offer(&self, offer_id: u64, offer: &Offer<Self::Api>) {
        if !offer.new_version {
            self.send().direct(
                &offer.offer_owner,
                &offer.payment_token_type,
                offer.payment_token_nonce,
                &offer.price,
            );
        }

        self.common_offer_remove(offer_id, offer);
        self.emit_withdraw_offer_event(offer_id, offer);
    }

    fn common_offer_auction_check(&self, offer: &Offer<Self::Api>, auction: &Auction<Self::Api>) {
        require!(
            auction.auction_type == AuctionType::Nft,
            "Cannot accept or decline offers for auctions, just for listings with a fixed price!"
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

        require!(
            offer.offer_owner != auction.original_owner,
            "Cannot accept your own offer!"
        );
    }

    fn common_offer_remove(&self, offer_id: u64, offer: &Offer<Self::Api>) {
        self.check_offer_sent(
            &offer.offer_owner,
            &offer.token_type,
            offer.token_nonce,
            &offer.payment_token_type,
        )
        .clear();
        self.token_offers_ids(&offer.token_type, offer.token_nonce)
            .remove(&offer_id);
        self.offers_by_wallet(&offer.offer_owner).remove(&offer_id);
        self.offer_by_id(offer_id).clear();
        self.offers().remove(&offer_id);
    }

    fn distribute_tokens_common(
        &self,
        nfts: ManagedVec<EsdtTokenPayment>,
        payment_token_id: &EgldOrEsdtTokenIdentifier,
        payment_token_nonce: u64,
        creator: &ManagedAddress,
        original_owner: &ManagedAddress,
        new_owner: &ManagedAddress,
        bid_split_amounts: &FeesDistribution<Self::Api>,
        wrapping: bool,
    ) {
        let total_amount = &bid_split_amounts.seller
            + &bid_split_amounts.creator
            + &bid_split_amounts.marketplace
            + &bid_split_amounts.extra;

        if bid_split_amounts.creator >= 0u64
            && total_amount >= MIN_TRADE_REWARD
            && payment_token_id.is_egld()
        {
            self.distribute_rewards(new_owner, original_owner);
        }

        if wrapping {
            if payment_token_id.is_egld() {
                self.unwrap_egld(total_amount);
            } else if payment_token_id.is_esdt() {
                self.wrap_egld(total_amount);
            }
        }

        if bid_split_amounts.extra > BigUint::zero()
            && bid_split_amounts.extra_address != ManagedAddress::zero()
        {
            self.transfer_or_save_payment(
                &bid_split_amounts.extra_address,
                payment_token_id,
                payment_token_nonce,
                &bid_split_amounts.extra,
            );
        }

        // send rest of the bid to original owner
        self.transfer_or_save_payment(
            original_owner,
            payment_token_id,
            payment_token_nonce,
            &bid_split_amounts.seller,
        );

        // send NFT to new owner
        self.tx().to(new_owner).multi_esdt(nfts).transfer_execute();

        if bid_split_amounts.reverse_cut_fees {
            self.transfer_or_save_payment(
                new_owner,
                payment_token_id,
                payment_token_nonce,
                &bid_split_amounts.marketplace,
            );
        } else {
            self.share_marketplace_fees(
                payment_token_id,
                bid_split_amounts.marketplace.clone(),
                payment_token_nonce,
            );
        }

        if bid_split_amounts.reverse_royalties {
            self.transfer_or_save_payment(
                new_owner,
                payment_token_id,
                payment_token_nonce,
                &bid_split_amounts.creator,
            );
        } else {
            // send part as royalties to creator
            self.share_royalties(
                creator,
                payment_token_id,
                bid_split_amounts.creator.clone(),
                payment_token_nonce,
            );
        }
    }

    fn distribute_tokens_bulk_buy(
        &self,
        payment_token_id: &EgldOrEsdtTokenIdentifier,
        payment_token_nonce: u64,
        creator: &ManagedAddress,
        original_owner: &ManagedAddress,
        new_owner: &ManagedAddress,
        bid_split_amounts: &FeesDistribution<Self::Api>,
        wrapping: bool,
    ) {
        let total_amount =
            &bid_split_amounts.seller + &bid_split_amounts.creator + &bid_split_amounts.marketplace;
        if total_amount >= MIN_TRADE_REWARD && payment_token_id.is_egld() {
            self.distribute_rewards(new_owner, original_owner);
        }

        if wrapping {
            if payment_token_id.is_egld() {
                // A platit cu WEGLD trebuie transformat in EGLD
                self.unwrap_egld(total_amount);
            } else if payment_token_id.is_esdt() {
                // A platit cu EGLD trebuie transformat in WEGLD
                self.wrap_egld(total_amount);
            }
        }

        if bid_split_amounts.extra > BigUint::zero()
            && bid_split_amounts.extra_address != ManagedAddress::zero()
        {
            self.transfer_or_save_payment(
                &bid_split_amounts.extra_address,
                payment_token_id,
                payment_token_nonce,
                &bid_split_amounts.extra,
            );
        }

        if bid_split_amounts.reverse_royalties {
            self.transfer_or_save_payment(
                new_owner,
                payment_token_id,
                payment_token_nonce,
                &bid_split_amounts.creator,
            );
        } else {
            // send part as royalties to creator
            self.share_royalties(
                creator,
                payment_token_id,
                bid_split_amounts.creator.clone(),
                payment_token_nonce,
            );
        }

        // send rest of the bid to original owner
        self.transfer_or_save_payment(
            original_owner,
            payment_token_id,
            payment_token_nonce,
            &bid_split_amounts.seller,
        );
    }

    fn share_marketplace_fees(
        &self,
        payment_token_id: &EgldOrEsdtTokenIdentifier,
        amount: BigUint,
        payment_token_nonce: u64,
    ) {
        if amount > BigUint::zero() {
            self.tx()
                .to(self.accumulator().get())
                .typed(accumulator::AccumulatorProxy)
                .deposit()
                .with_egld_or_single_esdt_transfer((
                    payment_token_id.clone(),
                    payment_token_nonce,
                    amount,
                ))
                .gas(5_500_000)
                .transfer_execute();
        }
    }

    fn share_royalties(
        &self,
        creator: &ManagedAddress,
        payment_token_id: &EgldOrEsdtTokenIdentifier,
        amount: BigUint,
        payment_token_nonce: u64,
    ) {
        if amount > BigUint::zero() {
            self.tx()
                .to(self.accumulator().get())
                .typed(accumulator::AccumulatorProxy)
                .deposit_royalties(creator)
                .egld_or_single_esdt(payment_token_id, payment_token_nonce, &amount)
                .gas(5_500_000)
                .transfer_execute();
        }
    }

    fn distribute_rewards(&self, buyer: &ManagedAddress, seller: &ManagedAddress) {
        let ticker_map = self.reward_ticker();
        if !ticker_map.is_empty() {
            let map_balance = self.reward_balance();
            let reward = self.reward_amount().get();
            let ticker = ticker_map.get();
            let balance_sc = self.blockchain().get_esdt_balance(
                &self.blockchain().get_sc_address(),
                &ticker.clone().into_esdt_option().unwrap(),
                0u64,
            );
            let reward_to_share = reward.clone().mul(2u64);
            if map_balance.get().ge(&reward_to_share) && balance_sc.ge(&reward_to_share) {
                self.transfer_or_save_payment(buyer, &ticker, 0u64, &reward);

                self.transfer_or_save_payment(seller, &ticker, 0u64, &reward);

                map_balance.update(|qt| *qt -= reward_to_share);
            }
        }
    }

    fn aggregate(
        &self,
        sent_to: &ManagedAddress,
        paid_by: &ManagedAddress,
        quantity: &BigUint,
        total_price: &BigUint,
        auction_id: u64,
        payment: EgldOrEsdtTokenPayment,
        steps: ManagedArgBuffer<Self::Api>,
        message: OptionalValue<ManagedBuffer>,
    ) {
        let gas_left = self.blockchain().get_gas_left();
        let callback = 30_000_000;
        // Execute swap via external router with source tokens and configuration
        self.tx()
            .to(self.aggregator_sc().get()) // Use protocol-configured swap router
            .raw_call(ManagedBuffer::new_from_bytes(b"xo"))
            .arguments_raw(steps) // Pass through swap configuration (path, slippage, etc.)
            .payment(&payment)
            .gas(gas_left - callback)
            .callback(self.callbacks().callback_ash(
                sent_to,
                paid_by,
                quantity,
                total_price,
                auction_id,
                payment.clone(),
                message,
            ))
            .gas_for_callback(30_000_000)
            .register_promise();
    }

    #[promises_callback]
    fn callback_ash(
        &self,
        send_to: &ManagedAddress,
        paid_by: &ManagedAddress,
        quantity: &BigUint,
        total_price: &BigUint,
        auction_id: u64,
        original_payment: EgldOrEsdtTokenPayment,
        message: OptionalValue<ManagedBuffer>,
    ) {
        self.freezed_auctions().swap_remove(&auction_id);
        let wegld = self.wrapping_token().get();
        let payment = self.call_value().egld_or_single_esdt();

        if payment.amount > 0 {
            let mut auction = self.try_get_auction(auction_id);
            let wrapping =
                self.require_egld_conversion(&auction, &payment.token_identifier, &wegld);
            let has_required_token =
                payment.token_identifier == auction.payment_token_type || wrapping;
            if &payment.amount >= total_price && has_required_token {
                let extra_amount = &payment.amount - total_price;
                self.transfer_or_save_payment(
                    paid_by,
                    &payment.token_identifier,
                    payment.token_nonce,
                    &extra_amount,
                );
                auction.current_winner = send_to.clone();
                auction.current_bid = total_price.clone();
                auction.nr_auctioned_tokens -= quantity;
                if auction.nr_auctioned_tokens == 0 {
                    self.remove_auction_common(auction_id, &auction);
                } else {
                    self.auction_by_id(auction_id).set(&auction);
                }
                self.update_or_remove_items_quantity(&auction, quantity);

                let current_time = self
                    .blockchain()
                    .get_block_timestamp_seconds()
                    .as_u64_seconds();
                self.emit_buy_event(
                    auction_id,
                    &auction,
                    quantity,
                    current_time,
                    message,
                    match paid_by == send_to {
                        true => OptionalValue::None,
                        false => OptionalValue::Some(paid_by.clone()),
                    },
                    &original_payment,
                );
                self.distribute_tokens(&auction, Option::Some(quantity), wrapping);
            } else {
                self.tx().to(paid_by).payment(&payment).transfer_execute();
            }
        }
    }
}
