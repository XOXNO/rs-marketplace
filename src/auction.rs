elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
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

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub enum AuctionType {
    None,
    NftBid,
    Nft,
    SftAll,
    SftOnePerPayment,
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
