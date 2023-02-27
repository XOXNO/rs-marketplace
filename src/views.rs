multiversx_sc::imports!();

use crate::auction::*;

#[multiversx_sc::module]
pub trait ViewsModule: crate::storage::StorageModule {
    #[view(getListingsCount)]
    fn get_listings_count(&self) -> usize {
        self.listings().len()
    }

    #[view(getOffersCount)]
    fn get_offers_count(&self) -> usize {
        self.offers().len()
    }

    #[view(getGlobalOffersCount)]
    fn get_global_offers_count(&self) -> usize {
        self.global_offer_ids().len()
    }

    #[view(getListings)]
    fn get_listings(&self, from: usize, to: usize) -> MultiValueEncoded<u64> {
        let mut results = MultiValueEncoded::new();

        for auction_id in self.listings().iter().skip(from).take(to) {
            results.push(auction_id);
        }

        return results;
    }

    #[view(getCollectionsCount)]
    fn get_collections_count(&self) -> usize {
        self.collections_listed().len()
    }

    #[view(isCollectionListed)]
    fn is_collection_listed(&self, token: TokenIdentifier) -> bool {
        self.collections_listed().contains(&token)
    }

    #[view(expiredOffersCount)]
    fn expired_offers_count(&self) -> i32 {
        let timestamp = self.blockchain().get_block_timestamp();
        let mut found = 0;
        for offer_id in self.offers().iter() {
            let offer = self.offer_by_id(offer_id).get();
            if offer.deadline < timestamp {
                found += 1;
            }
        }
        found
    }

    #[view(getAcceptedTokensCount)]
    fn get_accepted_tokens_count(&self) -> usize {
        self.accepted_tokens().len()
    }

    #[view(getTokenItemsForSaleCount)]
    fn get_token_items_for_sale_count(&self, token: &TokenIdentifier) -> usize {
        self.token_items_for_sale(token).len()
    }

    #[allow(clippy::too_many_arguments)]
    #[view(getOnSaleTokensForTicker)]
    fn get_on_sale_tokens_for_ticker(
        &self,
        token: &TokenIdentifier,
        nonces: MultiValueEncoded<u64>,
    ) -> ManagedVec<TokensOnSale<Self::Api>> {
        let mut results = ManagedVec::new();
        if self.token_items_for_sale(token).is_empty() {
            return results;
        }
        for nonce in nonces.into_iter() {
            let auctions = self.token_auction_ids(token, nonce);
            for auction in auctions.iter() {
                let auction_info = self.auction_by_id(auction).get();
                let token_type = self.blockchain().get_esdt_token_data(
                    &self.blockchain().get_owner_address(),
                    &auction_info.auctioned_token_type,
                    auction_info.auctioned_token_nonce,
                );
                let result = TokensOnSale {
                    auction_id: auction,
                    auction: auction_info,
                    token_type: token_type.token_type.as_u8(),
                };
                results.push(result);
            }
        }
        return results;
    }

    #[view(getAuctionsForTicker)]
    fn get_auctions_for_ticker(&self, token: &TokenIdentifier) -> ManagedVec<u64> {
        let mut results = ManagedVec::new();
        let nonces = self.token_items_for_sale(token);

        let timestamp = self.blockchain().get_block_timestamp();
        for nonce in nonces.iter() {
            let auctions = self.token_auction_ids(token, nonce);
            for auction_id in auctions.iter() {
                let auction_info = self.auction_by_id(auction_id);
                let dl = auction_info.get().deadline;
                if dl > timestamp || dl == 0 {
                    results.push(auction_id);
                }
            }
        }
        results
    }

    #[view(checkTokenOffers)]
    fn check_token_offers(
        &self,
        token: &TokenIdentifier,
        nonces: MultiValueEncoded<u64>,
    ) -> ManagedVec<BulkOffers<Self::Api>> {
        let mut results = ManagedVec::new();
        for nonce in nonces.into_iter() {
            let offers = self.token_offers_ids(token, nonce);
            if !offers.is_empty() {
                for offer_id in offers.iter() {
                    let offer_info = self.offer_by_id(offer_id).get();
                    let result = BulkOffers {
                        offer_id: offer_id,
                        offer: offer_info,
                        nonce: nonce,
                    };
                    results.push(result);
                }
            }
        }
        return results;
    }

    #[view(getBulkOffers)]
    fn get_bulk_offers(&self, offers: MultiValueEncoded<u64>) -> ManagedVec<BulkOffers<Self::Api>> {
        let mut results = ManagedVec::new();
        for offer_id in offers.into_iter() {
            if !self.offer_by_id(offer_id).is_empty() {
                let offer = self.offer_by_id(offer_id).get();
                let result = BulkOffers {
                    offer_id: offer_id,
                    nonce: offer.token_nonce,
                    offer: offer,
                };
                results.push(result);
            }
        }
        return results;
    }

    #[view(getBulkListings)]
    fn get_bulk_listings(
        &self,
        auction_ids: MultiValueEncoded<u64>,
    ) -> ManagedVec<TokensOnSale<Self::Api>> {
        let mut results = ManagedVec::new();
        for auction_id in auction_ids.into_iter() {
            if !self.auction_by_id(auction_id).is_empty() {
                let auction = self.auction_by_id(auction_id).get();
                let token_type = self.blockchain().get_esdt_token_data(
                    &self.blockchain().get_owner_address(),
                    &auction.auctioned_token_type,
                    auction.auctioned_token_nonce,
                );
                let result = TokensOnSale {
                    auction_id: auction_id,
                    auction,
                    token_type: token_type.token_type.as_u8(),
                };
                results.push(result);
            }
        }
        return results;
    }

    #[view(doesAuctionExist)]
    fn does_auction_exist(&self, auction_id: u64) -> bool {
        !self.auction_by_id(auction_id).is_empty()
    }

    #[view(doesGlobalOfferExist)]
    fn does_global_offer_exist(&self, auction_id: u64) -> bool {
        !self.global_offer(auction_id).is_empty()
    }

    #[view(doesOfferExist)]
    fn does_offer_exist(&self, offer_id: u64) -> bool {
        !self.offer_by_id(offer_id).is_empty()
    }

    #[view(isSCWl)]
    fn is_sc_wl(&self, sc: ManagedAddress) -> bool {
        self.whitelisted_contracts().contains(&sc)
    }

    #[view(getAuctionedToken)]
    fn get_auctioned_token(
        &self,
        auction_id: u64,
    ) -> OptionalValue<MultiValue3<TokenIdentifier, u64, BigUint>> {
        if self.does_auction_exist(auction_id) {
            let auction = self.auction_by_id(auction_id).get();

            OptionalValue::Some(
                (
                    auction.auctioned_token_type,
                    auction.auctioned_token_nonce,
                    auction.nr_auctioned_tokens,
                )
                    .into(),
            )
        } else {
            OptionalValue::None
        }
    }

    #[view(getAuctionedTokenAndOwner)]
    fn get_auctioned_token_and_owner(&self, auction_id: u64) -> OptionalValue<Auction<Self::Api>> {
        if self.does_auction_exist(auction_id) {
            let auction = self.auction_by_id(auction_id).get();
            OptionalValue::Some(auction)
        } else {
            OptionalValue::None
        }
    }

    #[endpoint(getAuctionType)]
    fn get_auction_type(&self, auction_id: u64) -> AuctionType {
        if self.does_auction_exist(auction_id) {
            self.auction_by_id(auction_id).get().auction_type
        } else {
            AuctionType::None
        }
    }

    #[view(getPaymentTokenForAuction)]
    fn get_payment_token_for_auction(
        &self,
        auction_id: u64,
    ) -> OptionalValue<MultiValue2<TokenIdentifier, u64>> {
        if self.does_auction_exist(auction_id) {
            let esdt_token = self.auction_by_id(auction_id).get();

            OptionalValue::Some(
                (
                    esdt_token.payment_token_type.into_esdt_option().unwrap(),
                    esdt_token.payment_token_nonce,
                )
                    .into(),
            )
        } else {
            OptionalValue::None
        }
    }

    #[view(getMinMaxBid)]
    fn get_min_max_bid(&self, auction_id: u64) -> OptionalValue<MultiValue2<BigUint, BigUint>> {
        if self.does_auction_exist(auction_id) {
            let auction = self.auction_by_id(auction_id).get();

            OptionalValue::Some(
                (
                    auction.min_bid,
                    auction.max_bid.unwrap_or_else(|| BigUint::zero()),
                )
                    .into(),
            )
        } else {
            OptionalValue::None
        }
    }

    #[view(getStartTime)]
    fn get_start_time(&self, auction_id: u64) -> OptionalValue<u64> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().start_time)
        } else {
            OptionalValue::None
        }
    }

    #[view(getDeadline)]
    fn get_deadline(&self, auction_id: u64) -> OptionalValue<u64> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().deadline)
        } else {
            OptionalValue::None
        }
    }

    #[view(getOriginalOwner)]
    fn get_original_owner(&self, auction_id: u64) -> OptionalValue<ManagedAddress> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().original_owner)
        } else {
            OptionalValue::None
        }
    }

    #[view(getCurrentWinningBid)]
    fn get_current_winning_bid(&self, auction_id: u64) -> OptionalValue<BigUint> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().current_bid)
        } else {
            OptionalValue::None
        }
    }

    #[view(getCurrentWinner)]
    fn get_current_winner(&self, auction_id: u64) -> OptionalValue<ManagedAddress> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().current_winner)
        } else {
            OptionalValue::None
        }
    }

    #[view(getFullAuctionData)]
    fn get_full_auction_data(&self, auction_id: u64) -> OptionalValue<Auction<Self::Api>> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get())
        } else {
            OptionalValue::None
        }
    }
}
