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
        // let nonces = self.token_items_for_sale(token.clone());
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

    #[view(getTokenBalanceDifference)]
    fn get_token_balance_difference(&self, token: TokenIdentifier, nonce: u64) -> BigUint {
        let local_balance = self.local_token_balance(token.clone()).get();
        let mut sc_balance = BigUint::zero();
        if (token.is_egld()) {
            sc_balance = self
                .blockchain()
                .get_balance(&self.blockchain().get_sc_address());
        } else if (token.is_valid_esdt_identifier() && token.is_esdt()) {
            sc_balance = self.blockchain().get_esdt_balance(
                &self.blockchain().get_sc_address(),
                &token,
                nonce,
            );
        }
        return sc_balance - local_balance;
    }

    #[view(doesAuctionExist)]
    fn does_auction_exist(&self, auction_id: u64) -> bool {
        !self.auction_by_id(auction_id).is_empty()
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
