elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Auction<M: ManagedTypeApi> {
    pub auctioned_token_type: TokenIdentifier<M>,
    pub auctioned_token_nonce: u64,
    pub nr_auctioned_tokens: BigUint<M>,
    pub auction_type: AuctionType,
    pub payment_token_type: TokenIdentifier<M>,
    pub payment_token_nonce: u64,
    pub min_bid: BigUint<M>,
    pub max_bid: Option<BigUint<M>>,
    pub start_time: u64,
    pub deadline: u64,

    pub original_owner: ManagedAddress<M>,
    pub current_bid: BigUint<M>,
    pub current_winner: ManagedAddress<M>,
    pub marketplace_cut_percentage: BigUint<M>,
    pub creator_royalties_percentage: BigUint<M>,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct Offer<M: ManagedTypeApi> {
    pub token_type: TokenIdentifier<M>,
    pub token_nonce: u64,
    pub quantity: BigUint<M>,
    pub status: OfferStatus,
    pub payment_token_type: TokenIdentifier<M>,
    pub payment_token_nonce: u64,
    pub price: BigUint<M>,
    pub deadline: u64,
    pub timestamp: u64,
    pub offer_owner: ManagedAddress<M>,
    pub marketplace_cut_percentage: BigUint<M>,
}

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct TokensOnSale<M: ManagedTypeApi> {
    pub auction: Auction<M>,
    pub auction_id: u64,
    pub token_type: u8,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct BulkOffers<M: ManagedTypeApi> {
    pub offer: Offer<M>,
    pub offer_id: u64,
    pub nonce: u64,
}

#[derive(
    ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug,
)]
pub enum AuctionType {
    None,
    NftBid,
    Nft,
    SftAll,
    SftOnePerPayment,
}

#[derive(
    ManagedVecItem,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    PartialEq,
    Debug,
    Clone,
)]
pub enum OfferStatus {
    Pending,
    Accepted,
    Declined,
    Withdraw,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct EsdtToken<M: ManagedTypeApi> {
    pub token_type: TokenIdentifier<M>,
    pub nonce: u64,
}

pub struct BidSplitAmounts<M: ManagedTypeApi> {
    pub creator: BigUint<M>,
    pub marketplace: BigUint<M>,
    pub seller: BigUint<M>,
}

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct P2PListedOffer<M: ManagedTypeApi> {
    pub offer_id: u64,
    pub owner: ManagedAddress<M>,
    pub accepted_tokens: ManagedVec<M, TokenIdentifier<M>>,
    pub mandatory_only: bool,
    pub offered_tokens: ManagedVec<M, EsdtTokenPayment<M>>,
    pub offers_linked: ManagedVec<M, u64>,
    pub reserved_for: Option<ManagedAddress<M>>,
    pub last_change: u64,
}

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct P2PCounterOffer<M: ManagedTypeApi> {
    pub visible: bool,
    pub offer_id: u64,
    pub owner: ManagedAddress<M>,
    pub offered_tokens: ManagedVec<M, EsdtTokenPayment<M>>,
    pub linked_offer: u64,
    pub status: P2POfferStatus,
    pub last_change: u64,
}

#[derive(
    ManagedVecItem,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    PartialEq,
    Debug,
    Clone,
)]
pub enum P2POfferStatus {
    Pending,
    Accepted,
    Declined,
    Withdraw,
}
