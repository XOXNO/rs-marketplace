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
    fn expired_offers_count(&self) -> ManagedVec<u64> {
        let timestamp = self.blockchain().get_block_timestamp_seconds().as_u64_seconds();
        let mut vc = ManagedVec::new();
        for offer_id in self.offers().iter() {
            let offer = self.offer_by_id(offer_id).get();
            if offer.deadline < timestamp {
                vc.push(offer_id);
            }
        }
        vc
    }

    #[view(getAcceptedTokensCount)]
    fn get_accepted_tokens_count(&self) -> usize {
        self.accepted_tokens().len()
    }

    #[view(getTokenItemsForSaleCount)]
    fn get_token_items_for_sale_count(&self, token: &TokenIdentifier) -> usize {
        self.token_items_for_sale(token).len()
    }

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

        for nonce in nonces.iter() {
            let auctions = self.token_auction_ids(token, nonce);
            for auction_id in auctions.iter() {
                results.push(auction_id);
            }
        }
        results
    }

    #[view(getFullAuctionsForTicker)]
    fn get_full_auctions_for_ticker(
        &self,
        token: &TokenIdentifier,
    ) -> ManagedVec<Auction<Self::Api>> {
        let mut results = ManagedVec::new();
        let nonces = self.token_items_for_sale(token);

        for nonce in nonces.iter() {
            let auctions = self.token_auction_ids(token, nonce);
            for auction_id in auctions.iter() {
                let auction_info = self.auction_by_id(auction_id).get();
                results.push(auction_info);
            }
        }
        results
    }

    #[view(getTokenOffers)]
    fn get_token_offers(
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
                        offer_id,
                        offer: offer_info,
                        nonce,
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
            let map = self.offer_by_id(offer_id);
            if !map.is_empty() {
                let offer = map.get();
                let result = BulkOffers {
                    offer_id,
                    nonce: offer.token_nonce,
                    offer,
                };
                results.push(result);
            }
        }
        return results;
    }

    #[view(getBulkGlobalOffers)]
    fn get_bulk_global_offers(
        &self,
        offers: MultiValueEncoded<u64>,
    ) -> ManagedVec<GlobalOffer<Self::Api>> {
        let mut results = ManagedVec::new();
        for offer_id in offers.into_iter() {
            let map_offer = self.global_offer(offer_id);
            if !map_offer.is_empty() {
                let offer = map_offer.get();
                results.push(offer);
            }
        }
        return results;
    }

    #[view(getBulkGlobalOffersByCollection)]
    fn get_bulk_global_offers_by_collection(
        &self,
        ticker: TokenIdentifier,
    ) -> ManagedVec<GlobalOffer<Self::Api>> {
        let mut results = ManagedVec::new();
        let offers = self.collection_global_offers(&ticker);
        for offer_id in offers.into_iter() {
            let map = self.global_offer(offer_id);
            if !map.is_empty() {
                let offer = map.get();
                results.push(offer);
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
            let map = self.auction_by_id(auction_id);
            if !map.is_empty() {
                let auction = map.get();
                let token_type = self.blockchain().get_esdt_token_data(
                    &self.blockchain().get_owner_address(),
                    &auction.auctioned_token_type,
                    auction.auctioned_token_nonce,
                );
                let result = TokensOnSale {
                    auction_id,
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
}
