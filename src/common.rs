multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use crate::{
    auction::{
        AggregatorStep, Auction, AuctionType, FeesDistribution, GlobalOffer, Offer, TokenAmount,
    },
    NFT_AMOUNT,
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
        self.update_or_remove_items_quantity(&auction, &auction.nr_auctioned_tokens);
        self.remove_auction_common(auction_id, &auction);
        self.emit_end_auction_event(auction_id, auction);
        self.distribute_tokens(&auction, Option::Some(&auction.nr_auctioned_tokens), false);
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
        let current_time = self.blockchain().get_block_timestamp();
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
        swaps: OptionalValue<ManagedVec<AggregatorStep<Self::Api>>>,
        limits: OptionalValue<MultiValueEncoded<TokenAmount<Self::Api>>>,
    ) {
        require!(self.status().get(), "Global operation enabled!");
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
        let require_swap = swaps.is_some();
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

            let current_time = self.blockchain().get_block_timestamp();
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
            let mut limits = limits.into_option().unwrap();
            limits.push(TokenAmount::new(
                match auction.payment_token_type.is_egld() {
                    true => self.wrapping_token().get(),
                    false => auction.payment_token_type.unwrap_esdt(),
                },
                total_value.clone(),
            ));
            self.freezed_auctions().insert(auction_id);
            let gas_left = self.blockchain().get_gas_left();
            let req_gas = (20_000_000 + steps.len() * 15_000_000).try_into().unwrap();
            require!(
                gas_left >= req_gas,
                "Not enough gas left to complete the transaction!"
            );
            self.aggregate(
                buyer,
                &caller,
                &buy_amount,
                &total_value,
                auction_id,
                payments,
                req_gas,
                steps,
                limits,
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
                &EgldOrEsdtTokenIdentifier::esdt(auction.auctioned_token_type.clone()),
                auction.auctioned_token_nonce,
                nft_amount_to_send,
                &auction.payment_token_type,
                auction.payment_token_nonce,
                &nft_info.creator,
                &auction.original_owner,
                &auction.current_winner,
                &bid_split_amounts,
                wrapping,
            );
        } else {
            self.return_auction_nft(&auction);
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

        if map_token_for_sale.len() == 0 {
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
        nft_type: &EgldOrEsdtTokenIdentifier,
        nft_nonce: u64,
        nft_amount_to_send: &BigUint,
        payment_token_id: &EgldOrEsdtTokenIdentifier,
        payment_token_nonce: u64,
        creator: &ManagedAddress,
        original_owner: &ManagedAddress,
        new_owner: &ManagedAddress,
        bid_split_amounts: &FeesDistribution<Self::Api>,
        wrapping: bool,
    ) {
        // send part as cut for contract owner
        let wegld = self.wrapping_token().get();
        if wrapping {
            if payment_token_id.is_egld() {
                self.unwrap_egld(
                    &bid_split_amounts.seller
                        + &bid_split_amounts.creator
                        + &bid_split_amounts.marketplace
                        + &bid_split_amounts.extra,
                );
            } else if payment_token_id.is_esdt() {
                self.wrap_egld(
                    &bid_split_amounts.seller
                        + &bid_split_amounts.creator
                        + &bid_split_amounts.marketplace
                        + &bid_split_amounts.extra,
                );
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
            self.transfer_or_save_payment(
                creator,
                payment_token_id,
                payment_token_nonce,
                &bid_split_amounts.creator,
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
        self.transfer_or_save_payment(new_owner, nft_type, nft_nonce, nft_amount_to_send);
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
                wegld,
                wrapping,
            );
        }
        self.distribute_rewards(new_owner, original_owner);
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
        if wrapping {
            if payment_token_id.is_egld() {
                // A platit cu WEGLD trebuie transformat in EGLD
                self.unwrap_egld(
                    &bid_split_amounts.seller
                        + &bid_split_amounts.creator
                        + &bid_split_amounts.marketplace,
                );
            } else if payment_token_id.is_esdt() {
                // A platit cu EGLD trebuie transformat in WEGLD
                self.wrap_egld(
                    &bid_split_amounts.seller
                        + &bid_split_amounts.creator
                        + &bid_split_amounts.marketplace,
                );
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
            self.transfer_or_save_payment(
                creator,
                payment_token_id,
                payment_token_nonce,
                &bid_split_amounts.creator,
            );
        }

        // send rest of the bid to original owner
        self.transfer_or_save_payment(
            original_owner,
            payment_token_id,
            payment_token_nonce,
            &bid_split_amounts.seller,
        );
        self.distribute_rewards(new_owner, original_owner);
    }

    fn share_marketplace_fees(
        &self,
        payment_token_id: &EgldOrEsdtTokenIdentifier,
        amount: BigUint,
        payment_token_nonce: u64,
        _wegld: TokenIdentifier,
        _wrapping: bool,
    ) {
        let sc_owner = self.blockchain().get_owner_address();
        // if payment_token_id.is_egld() ||  payment_token_id.eq(&wegld) {
        //     if !wrapping && payment_token_id.is_egld() {
        //         self.wrap_egld(amount.clone());
        //     }
        //     self.swap_wegld_for_xoxno(&sc_owner, EsdtTokenPayment::new(wegld, 0, amount));
        // } else {
        self.transfer_or_save_payment(&sc_owner, payment_token_id, payment_token_nonce, &amount);
        // }
    }

    fn distribute_rewards(&self, buyer: &ManagedAddress, seller: &ManagedAddress) {
        let ticker_map = self.reward_ticker();
        if !ticker_map.is_empty() {
            let map_balance = self.reward_balance();
            let reward = self.reward_amount().get();
            let ticker = ticker_map.get();
            let balance_sc = self.blockchain().get_esdt_balance(&self.blockchain().get_sc_address(), &ticker.clone().into_esdt_option().unwrap(), 0u64);
            let reward_to_share = reward.clone().mul(2u64);
            if map_balance.get().ge(&reward_to_share) && balance_sc.ge(&reward_to_share){
                self.transfer_or_save_payment(&buyer, &ticker, 0u64, &reward);

                self.transfer_or_save_payment(&seller, &ticker, 0u64, &reward);

                map_balance.update(|qt| *qt -= reward_to_share);
            }
        }
    }

    #[proxy]
    fn dex_proxy(&self, sc_address: ManagedAddress) -> ash_proxy::Proxy<Self::Api>;

    fn aggregate(
        &self,
        sent_to: &ManagedAddress,
        paid_by: &ManagedAddress,
        quantity: &BigUint,
        total_price: &BigUint,
        auction_id: u64,
        payment: EgldOrEsdtTokenPayment,
        gas: u64,
        steps: ManagedVec<AggregatorStep<Self::Api>>,
        limits: MultiValueEncoded<TokenAmount<Self::Api>>,
        message: OptionalValue<ManagedBuffer>,
    ) {
        let final_payment;
        if payment.token_identifier.is_egld() {
            self.wrap_egld(payment.amount.clone());
            final_payment =
                EsdtTokenPayment::new(self.wrapping_token().get(), 0, payment.amount.clone());
        } else {
            final_payment = payment.clone().unwrap_esdt();
        }
        let mut payments = ManagedVec::new();
        payments.push(final_payment);
        self.dex_proxy(self.aggregator_sc().get())
            .aggregate(steps, limits)
            .with_multi_token_transfer(payments)
            .with_gas_limit(gas)
            .async_call_promise()
            .with_callback(self.callbacks().callback_ash(
                sent_to,
                paid_by,
                quantity,
                total_price,
                auction_id,
                payment.clone(),
                message,
            ))
            .with_extra_gas_for_callback(30_000_000)
            .register_promise()
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
        let p = self.call_value().all_esdt_transfers();
        let payments = p.clone_value();
        if payments.len() > 0 {
            let payment = p.clone_value().get(0);
            let mut auction = self.try_get_auction(auction_id);
            let token = &EgldOrEsdtTokenIdentifier::esdt(payment.token_identifier);
            let wrapping = self.require_egld_conversion(&auction, token, &wegld);
            let has_required_token = token == &auction.payment_token_type || wrapping;
            if &payment.amount >= total_price && has_required_token {
                let extra_amount = &payment.amount - total_price;
                self.transfer_or_save_payment(paid_by, token, payment.token_nonce, &extra_amount);
                auction.current_winner = send_to.clone();
                auction.current_bid = total_price.clone();
                auction.nr_auctioned_tokens -= quantity;
                if auction.nr_auctioned_tokens == 0 {
                    self.remove_auction_common(auction_id, &auction);
                } else {
                    self.auction_by_id(auction_id).set(&auction);
                }
                self.update_or_remove_items_quantity(&auction, quantity);

                let current_time = self.blockchain().get_block_timestamp();
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
                self.send().direct_multi(paid_by, &payments);
            }
        } else {
            self.send().direct_multi(paid_by, &payments);
        }
    }
}

mod ash_proxy {
    multiversx_sc::imports!();
    use crate::auction::*;
    #[multiversx_sc::proxy]
    pub trait AshContract {
        #[payable("*")]
        #[endpoint]
        fn aggregate(
            &self,
            steps: ManagedVec<AggregatorStep<Self::Api>>,
            limits: MultiValueEncoded<TokenAmount<Self::Api>>,
        ) -> ManagedVec<EsdtTokenPayment>;
    }
}
