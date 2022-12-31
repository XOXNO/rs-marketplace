use crate::{
    auction::{Auction, AuctionType, BidSplitAmounts, GlobalOffer, Offer},
    NFT_AMOUNT,
};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait CommonModule:
    crate::storage::StorageModule
    + crate::helpers::HelpersModule
    + crate::views::ViewsModule
    + crate::events::EventsModule
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

    fn end_auction_common(&self, auction_id: u64, auction: &Auction<Self::Api>, current_time: u64) {
        self.distribute_tokens(&auction, Option::Some(&auction.nr_auctioned_tokens));
        self.update_or_remove_items_quantity(&auction, &auction.nr_auctioned_tokens);
        self.remove_auction_common(auction_id, &auction);
        self.emit_end_auction_event(auction_id, auction, current_time);
    }

    fn common_bid_checks(
        &self,
        auction: &Auction<Self::Api>,
        nft_type: &TokenIdentifier,
        nft_nonce: u64,
        payment_token: &EgldOrEsdtTokenIdentifier,
        payment_nonce: u64,
        payment_amount: &BigUint,
    ) {
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();

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

        require!(
            payment_token == &auction.payment_token_type
                && payment_nonce == auction.payment_token_nonce,
            "Wrong token used as payment"
        );
    }

    fn distribute_tokens(&self, auction: &Auction<Self::Api>, opt_sft_amount: Option<&BigUint>) {
        let nft_type = &auction.auctioned_token_type;
        let nft_nonce = auction.auctioned_token_nonce;
        if !auction.current_winner.is_zero() {
            let nft_info = self.get_nft_info(nft_type, nft_nonce);
            let bid_split_amounts = self.calculate_winning_bid_split(auction);

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
                &EgldOrEsdtTokenIdentifier::esdt(nft_type.clone()),
                nft_nonce,
                nft_amount_to_send,
                &auction.payment_token_type,
                auction.payment_token_nonce,
                &nft_info.creator,
                &auction.original_owner,
                &auction.current_winner,
                &bid_split_amounts,
            );
        } else {
            self.return_auction_nft(&auction);
        }
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
        if return_offer {
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
    }

    fn common_withdraw_offer(&self, offer_id: u64, offer: &Offer<Self::Api>) {
        self.send().direct(
            &offer.offer_owner,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &offer.price,
        );

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
        bid_split_amounts: &BidSplitAmounts<Self::Api>,
    ) {
        // send part as cut for contract owner
        let sc_owner = self.blockchain().get_owner_address();
        self.transfer_or_save_payment(
            &sc_owner,
            payment_token_id,
            payment_token_nonce,
            &bid_split_amounts.marketplace,
        );

        // send part as royalties to creator
        self.transfer_or_save_payment(
            creator,
            payment_token_id,
            payment_token_nonce,
            &bid_split_amounts.creator,
        );

        // send rest of the bid to original owner
        self.transfer_or_save_payment(
            original_owner,
            payment_token_id,
            payment_token_nonce,
            &bid_split_amounts.seller,
        );

        // send NFT to new owner
        self.transfer_or_save_payment(new_owner, nft_type, nft_nonce, nft_amount_to_send);
    }
}
