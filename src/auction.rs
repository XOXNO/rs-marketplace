use multiversx_sc::codec::NestedDecodeInput;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

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

#[derive(ManagedVecItem, TopEncode, NestedEncode, NestedDecode, TypeAbi, Clone)]
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
    pub new_version: bool,
}
impl<M: ManagedTypeApi> TopDecode for Offer<M> {
    fn top_decode<I>(input: I) -> Result<Self, DecodeError>
    where
        I: multiversx_sc::codec::TopDecodeInput,
    {
        let mut input = input.into_nested_buffer();
        let token_type = TokenIdentifier::dep_decode(&mut input)?;
        let token_nonce = u64::dep_decode(&mut input)?;
        let quantity = BigUint::dep_decode(&mut input)?;
        let status = OfferStatus::dep_decode(&mut input)?;
        let payment_token_type = EgldOrEsdtTokenIdentifier::dep_decode(&mut input)?;
        let payment_token_nonce = u64::dep_decode(&mut input)?;
        let price = BigUint::dep_decode(&mut input)?;
        let deadline = u64::dep_decode(&mut input)?;
        let timestamp = u64::dep_decode(&mut input)?;
        let offer_owner =  ManagedAddress::dep_decode(&mut input)?;
        let marketplace_cut_percentage = BigUint::dep_decode(&mut input)?;

        let new_version = if input.is_depleted() {
            false
        } else {
            bool::dep_decode(&mut input)?
        };

        Result::Ok(Offer {
            token_type,
            token_nonce,
            quantity,
            status,
            payment_token_type,
            payment_token_nonce,
            price,
            deadline,
            timestamp,
            offer_owner,
            marketplace_cut_percentage,
            new_version
        })
    }
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

pub struct FeesDistribution<M: ManagedTypeApi> {
    pub creator: BigUint<M>,
    pub marketplace: BigUint<M>,
    pub seller: BigUint<M>,
    pub extra: BigUint<M>,
    pub extra_address: ManagedAddress<M>,
    pub reverse_royalties: bool,
    pub reverse_cut_fee: bool,
}

#[derive(ManagedVecItem, TypeAbi, NestedEncode, NestedDecode, Clone, TopEncode)]
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
    pub new_version: bool,
}

impl<M: ManagedTypeApi> TopDecode for GlobalOffer<M> {
    fn top_decode<I>(input: I) -> Result<Self, DecodeError>
    where
        I: multiversx_sc::codec::TopDecodeInput,
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


        let new_version = if input.is_depleted() {
            false
        } else {
            bool::dep_decode(&mut input)?
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
            new_version
        })
    }
}

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct BulkListing<M: ManagedTypeApi> {
    pub min_bid: BigUint<M>,
    pub max_bid: BigUint<M>,
    pub deadline: u64,
    pub accepted_payment_token: EgldOrEsdtTokenIdentifier<M>,
    pub bid: bool,
    pub opt_sft_max_one_per_payment: bool,
    pub opt_start_time: u64,
    pub collection: EgldOrEsdtTokenIdentifier<M>,
    pub nonce: u64,
    pub nft_amount: BigUint<M>,
}

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct BulkUpdateListing<M: ManagedTypeApi> {
    pub payment_token_type: EgldOrEsdtTokenIdentifier<M>,
    pub new_price: BigUint<M>,
    pub auction_id: u64,
    pub deadline: u64,
}

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct CollectionFeeConfig<M: ManagedTypeApi> {
    pub reverse_cut_fees: bool,
    pub reverse_royalties: bool,
    pub custom_royalties: bool,
    pub min_royalties: BigUint<M>,
    pub max_royalties: BigUint<M>,
    pub extra_fees: CollectionExtraFeesConfig<M>,
    pub admin: ManagedAddress<M>, 
}

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct CollectionExtraFeesConfig<M: ManagedTypeApi> {
    pub amount: BigUint<M>,
    pub address: ManagedAddress<M>,
}
