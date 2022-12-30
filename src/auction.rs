elrond_wasm::imports!();
elrond_wasm::derive_imports!();
use elrond_wasm::elrond_codec::NestedDecodeInput;

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Auction<M: ManagedTypeApi> {
    pub auctioned_token_type: TokenIdentifier<M>,
    pub auctioned_token_nonce: u64,
    pub nr_auctioned_tokens: BigUint<M>,
    pub auction_type: AuctionType,
    pub payment_token_type: EgldOrEsdtTokenIdentifier<M>,
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
    pub payment_token_type: EgldOrEsdtTokenIdentifier<M>,
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
    Copy,
)]
pub enum OfferStatus {
    Pending,
    Accepted,
    Declined,
    Withdraw,
}

pub struct BidSplitAmounts<M: ManagedTypeApi> {
    pub creator: BigUint<M>,
    pub marketplace: BigUint<M>,
    pub seller: BigUint<M>,
}

#[derive(TopEncode, TypeAbi)]
pub struct GlobalOffer<M: ManagedTypeApi> {
    pub offer_id: u64,
    pub collection: TokenIdentifier<M>,
    pub quantity: BigUint<M>,
    pub payment_token: EgldOrEsdtTokenIdentifier<M>,
    pub payment_nonce: u64,
    pub price: BigUint<M>,
    pub timestamp: u64,
    pub owner: ManagedAddress<M>,
    pub attributes: Option<ManagedBuffer<M>>,
}

impl<M: ManagedTypeApi> TopDecode for GlobalOffer<M> {
    fn top_decode<I>(input: I) -> Result<Self, DecodeError>
    where
        I: elrond_codec::TopDecodeInput,
    {
        let mut input = input.into_nested_buffer();
        let offer_id = u64::dep_decode(&mut input)?;
        let collection = TokenIdentifier::dep_decode(&mut input)?;
        let quantity = BigUint::dep_decode(&mut input)?;
        let payment_token = EgldOrEsdtTokenIdentifier::dep_decode(&mut input)?;
        let payment_nonce = u64::dep_decode(&mut input)?;
        let price = BigUint::dep_decode(&mut input)?;
        let timestamp = u64::dep_decode(&mut input)?;
        let owner = ManagedAddress::dep_decode(&mut input)?;

        let attributes = if input.is_depleted() {
            None
        } else {
            Option::<ManagedBuffer<M>>::dep_decode(&mut input)?
        };

        Result::Ok(GlobalOffer {
            offer_id,
            collection,
            quantity,
            payment_token,
            payment_nonce,
            price,
            timestamp,
            owner,
            attributes,
        })
    }
}
