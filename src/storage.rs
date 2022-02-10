elrond_wasm::imports!();

use crate::auction::*;

#[elrond_wasm::module]
pub trait StorageModule {
    #[view(getMarketplaceCutPercentage)]
    #[storage_mapper("bidCutPercentage")]
    fn bid_cut_percentage(&self) -> SingleValueMapper<BigUint>;

    #[view(getOffersByWallet)]
    #[storage_mapper("offersByWallet")]
    fn offers_by_wallet(&self, address: ManagedAddress) -> SetMapper<u64>;

    #[view(checkOfferSent)]
    #[storage_mapper("checkOfferSent")]
    fn check_offer_sent(&self, address: ManagedAddress, nft: TokenIdentifier, nonce: u64, payment_token: TokenIdentifier) -> SingleValueMapper<bool>;

    #[view(getListingsByWallet)]
    #[storage_mapper("listingsByWallet")]
    fn listings_by_wallet(&self, address: ManagedAddress) -> SetMapper<u64>;

    #[view(getActiveListingsBids)]
    #[storage_mapper("listingsBids")]
    fn listings_bids(&self, address: ManagedAddress) -> SetMapper<u64>;

    #[view(getTokenItemsForSale)]
    #[storage_mapper("tokenItemsForSale")]
    fn token_items_for_sale(&self, token: TokenIdentifier) -> SetMapper<u64>;

    #[view(getTokenAuctionIds)]
    #[storage_mapper("tokenAuctionIDs")]
    fn token_auction_ids(&self, token: TokenIdentifier, nonce: u64) -> SetMapper<u64>;

    #[view(getTokenOffersIds)]
    #[storage_mapper("tokenOffersIDs")]
    fn token_offers_ids(&self, token: TokenIdentifier, nonce: u64) -> SetMapper<u64>;

    #[view(getTokenItemsQuantityForSale)]
    #[storage_mapper("tokenItemsQuantityForSale")]
    fn token_items_quantity_for_sale(
        &self,
        token: TokenIdentifier,
        nonce: u64,
    ) -> SingleValueMapper<BigUint>;

    #[view(getAcceptedTokens)]
    #[storage_mapper("acceptedTokens")]
    fn accepted_tokens(&self) -> SetMapper<TokenIdentifier>;


    #[view(blackListWallets)]
    #[storage_mapper("blacklistWallets")]
    fn blacklist_wallets(&self) -> SetMapper<ManagedAddress>;

    #[view(whitelistedContracts)]
    #[storage_mapper("whitelistedContracts")]
    fn whitelisted_contracts(&self) -> SetMapper<ManagedAddress>;

    #[view(getClaimableAmount)]
    #[storage_mapper("claimableAmount")]
    fn claimable_amount(
        &self,
        address: &ManagedAddress,
        token_id: &TokenIdentifier,
        token_nonce: u64,
    ) -> SingleValueMapper<BigUint>;

    #[view(getCollectionsListed)]
    #[storage_mapper("collectionsListed")]
    fn collections_listed(&self) -> SetMapper<TokenIdentifier>;

    #[view(getListings)]
    #[storage_mapper("listings")]
    fn listings(&self) -> SetMapper<u64>;

    #[view(getOffers)]
    #[storage_mapper("offers")]
    fn offers(&self) -> SetMapper<u64>;

    #[view(getStatus)]
    #[storage_mapper("status")]
    fn status(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("auctionById")]
    fn auction_by_id(&self, auction_id: u64) -> SingleValueMapper<Auction<Self::Api>>;

    #[view(offerById)]
    #[storage_mapper("offerById")]
    fn offer_by_id(&self, offer_id: u64) -> SingleValueMapper<Offer<Self::Api>>;

    #[view(getLastValidAuctionId)]
    #[storage_mapper("lastValidAuctionId")]
    fn last_valid_auction_id(&self) -> SingleValueMapper<u64>;

    #[view(getRewardBalance)]
    #[storage_mapper("getRewardBalance")]
    fn reward_balance(&self) -> SingleValueMapper<BigUint>;

    #[view(getRewardTicker)]
    #[storage_mapper("getRewardTicker")]
    fn reward_ticker(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(specialRewardAmount)]
    #[storage_mapper("specialRewardAmount")]
    fn special_reward_amount(&self, token: TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(defaultRewardAmount)]
    #[storage_mapper("defaultRewardAmount")]
    fn reward_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getLastValidOfferId)]
    #[storage_mapper("lastValidOfferId")]
    fn last_valid_offer_id(&self) -> SingleValueMapper<u64>;
}
