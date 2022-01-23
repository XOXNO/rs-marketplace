elrond_wasm::imports!();

use crate::auction::*;

#[elrond_wasm::module]
pub trait ViewsModule: crate::storage::StorageModule {
    #[view(getListingsCount)]
    fn get_listings_count(&self) -> usize {
        self.listings().len()
    }

    #[view(getCollectionsCount)]
    fn get_collections_count(&self) -> usize {
        self.collections_listed().len()
    }

    #[view(getAcceptedTokensCount)]
    fn get_accepted_tokens_count(&self) -> usize {
        self.accepted_tokens().len()
    }

    #[view(getTokenItemsForSaleCount)]
    fn get_token_items_for_sale_count(&self, token: TokenIdentifier) -> usize {
        self.token_items_for_sale(token).len()
    }

    #[allow(clippy::too_many_arguments)]
    #[view(getOnSaleTokensForTicker)]
    fn get_on_sale_tokens_for_ticker(&self, token: TokenIdentifier,  #[var_args] nonces: MultiArgVec<u64>) -> Vec<TokensOnSale<Self::Api>> {
        let mut results = Vec::new();
        if (self.token_items_for_sale(token.clone()).is_empty()) {
            return results;
        }
        for nonce in nonces.iter() {
            let auctions = self.token_auction_ids(token.clone(), *nonce);
            for auction in auctions.iter() {
                let auction_info = self.auction_by_id(auction).get();
                let result = TokensOnSale {
                    auction_id: auction,
                    auction: auction_info,
                };
                results.push(result);
            }
        }
        return results;
    }

    #[allow(clippy::too_many_arguments)]
    #[view(checkTokenOffers)] 
    fn check_token_offers(&self, token: TokenIdentifier,  #[var_args] nonces: MultiArgVec<u64>) -> Vec<BulkOffers<Self::Api>> {
        let mut results = Vec::new();
        for nonce in nonces.iter() {
            let offers = self.token_offers_ids(token.clone(), *nonce);
            if (!offers.is_empty()) {
                for offer_id in offers.iter() {
                    let offer_info = self.offer_by_id(offer_id).get();
                    let result = BulkOffers {
                        offer_id: offer_id,
                        offer: offer_info,
                        nonce: *nonce,
                    };
                    results.push(result);
                }
            }
        }
        return results;
    }
    #[allow(clippy::too_many_arguments)]
    #[view(getBulkOffers)]
    fn get_bulk_offers(&self, #[var_args] offers: MultiArgVec<u64>) -> Vec<BulkOffers<Self::Api>> {        
        let mut results = Vec::new();
        for offer_id in offers.iter() {
            if (!self.offer_by_id(*offer_id).is_empty()) {
                let offer = self.offer_by_id(*offer_id).get();
                let result = BulkOffers {
                    offer_id: *offer_id,
                    nonce: offer.token_nonce.clone(),
                    offer: offer,
                };
                results.push(result);
            }
        }
        return results;
    }

    #[allow(clippy::too_many_arguments)]
    #[view(getBulkListings)]
    fn get_bulk_listings(&self, #[var_args] auction_ids: MultiArgVec<u64>) -> Vec<TokensOnSale<Self::Api>> {        
        let mut results = Vec::new();
        for auction_id in auction_ids.iter() {
            if (!self.auction_by_id(*auction_id).is_empty()) {
                let auction = self.auction_by_id(*auction_id).get();
                let result = TokensOnSale {
                    auction_id: *auction_id,
                    auction,
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

    #[view(doesOfferExist)]
    fn does_offer_exist(&self, offer_id: u64) -> bool {
        !self.offer_by_id(offer_id).is_empty()
    }

    #[view(getAuctionedToken)]
    fn get_auctioned_token(
        &self,
        auction_id: u64,
    ) -> OptionalResult<MultiResult3<TokenIdentifier, u64, BigUint>> {
        if self.does_auction_exist(auction_id) {
            let auction = self.auction_by_id(auction_id).get();

            OptionalResult::Some(
                (
                    auction.auctioned_token_type,
                    auction.auctioned_token_nonce,
                    auction.nr_auctioned_tokens,
                )
                    .into(),
            )
        } else {
            OptionalResult::None
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
    ) -> OptionalResult<MultiResult2<TokenIdentifier, u64>> {
        if self.does_auction_exist(auction_id) {
            let esdt_token = self.auction_by_id(auction_id).get();

            OptionalResult::Some(
                (
                    esdt_token.payment_token_type,
                    esdt_token.payment_token_nonce,
                )
                    .into(),
            )
        } else {
            OptionalResult::None
        }
    }

    #[view(getMinMaxBid)]
    fn get_min_max_bid(&self, auction_id: u64) -> OptionalResult<MultiResult2<BigUint, BigUint>> {
        if self.does_auction_exist(auction_id) {
            let auction = self.auction_by_id(auction_id).get();

            OptionalResult::Some(
                (
                    auction.min_bid,
                    auction.max_bid.unwrap_or_else(|| BigUint::zero()),
                )
                    .into(),
            )
        } else {
            OptionalResult::None
        }
    }

    #[view(getStartTime)]
    fn get_start_time(&self, auction_id: u64) -> OptionalResult<u64> {
        if self.does_auction_exist(auction_id) {
            OptionalResult::Some(self.auction_by_id(auction_id).get().start_time)
        } else {
            OptionalResult::None
        }
    }

    #[view(getDeadline)]
    fn get_deadline(&self, auction_id: u64) -> OptionalResult<u64> {
        if self.does_auction_exist(auction_id) {
            OptionalResult::Some(self.auction_by_id(auction_id).get().deadline)
        } else {
            OptionalResult::None
        }
    }

    #[view(getOriginalOwner)]
    fn get_original_owner(&self, auction_id: u64) -> OptionalResult<ManagedAddress> {
        if self.does_auction_exist(auction_id) {
            OptionalResult::Some(self.auction_by_id(auction_id).get().original_owner)
        } else {
            OptionalResult::None
        }
    }

    #[view(getCurrentWinningBid)]
    fn get_current_winning_bid(&self, auction_id: u64) -> OptionalResult<BigUint> {
        if self.does_auction_exist(auction_id) {
            OptionalResult::Some(self.auction_by_id(auction_id).get().current_bid)
        } else {
            OptionalResult::None
        }
    }

    #[view(getCurrentWinner)]
    fn get_current_winner(&self, auction_id: u64) -> OptionalResult<ManagedAddress> {
        if self.does_auction_exist(auction_id) {
            OptionalResult::Some(self.auction_by_id(auction_id).get().current_winner)
        } else {
            OptionalResult::None
        }
    }

    #[view(getFullAuctionData)]
    fn get_full_auction_data(&self, auction_id: u64) -> OptionalResult<Auction<Self::Api>> {
        if self.does_auction_exist(auction_id) {
            OptionalResult::Some(self.auction_by_id(auction_id).get())
        } else {
            OptionalResult::None
        }
    }
}
