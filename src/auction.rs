elrond_wasm::imports!();
elrond_wasm::derive_imports!();


#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug, Clone)]
pub enum CustomOptionId {
    _ReservedNone,
    _ReservedSome,
    None,
    Some,
}

#[derive(ManagedVecItem, Clone, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct CustomOption<M: ManagedTypeApi> {
    pub id: CustomOptionId,
    pub value: BigUint<M>
}

#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode)]
pub struct Auction<M: ManagedTypeApi> {
    pub auctioned_token_type: TokenIdentifier<M>,
    pub auctioned_token_nonce: u64,
    pub nr_auctioned_tokens: BigUint<M>,
    pub auction_type: AuctionType,
    pub payment_token_type: TokenIdentifier<M>,
    pub payment_token_nonce: u64,
    pub min_bid: BigUint<M>,
    pub max_bid: CustomOption<M>,
    pub start_time: u64,
    pub deadline: u64,

    pub original_owner: ManagedAddress<M>,
    pub current_bid: BigUint<M>,
    pub current_winner: ManagedAddress<M>,
    pub marketplace_cut_percentage: BigUint<M>,
    pub creator_royalties_percentage: BigUint<M>,
}

impl<M: ManagedTypeApi> NestedDecode for Auction<M> {
    const TYPE_INFO: elrond_codec::TypeInfo = elrond_codec::TypeInfo::Unknown;

    fn dep_decode<I: elrond_codec::NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        Self::dep_decode_or_handle_err(input, elrond_codec::DefaultErrorHandler)
    }

    fn dep_decode_or_handle_err<I, H>(input: &mut I, h: H) -> Result<Self, H::HandledErr>
    where
        I: elrond_codec::NestedDecodeInput,
        H: elrond_codec::DecodeErrorHandler,
    {
        let auctioned_token_type = TokenIdentifier::dep_decode(input).unwrap();
        let auctioned_token_nonce = u64::dep_decode(input).unwrap();
        let nr_auctioned_tokens = BigUint::dep_decode(input).unwrap();
        let auction_type = AuctionType::dep_decode(input).unwrap();
        let payment_token_type = TokenIdentifier::dep_decode(input).unwrap();
        let payment_token_nonce = u64::dep_decode(input).unwrap();
        let min_bid = BigUint::dep_decode(input).unwrap();

        let option_prefix = u8::dep_decode(input).unwrap();
        let max_bid = match option_prefix {
            0 => CustomOption {
                id: CustomOptionId::None,
                value: BigUint::zero(),
            },
            1 => CustomOption {
                id: CustomOptionId::Some,
                value: BigUint::dep_decode(input).unwrap(),
            },
            2 => CustomOption {
                id: CustomOptionId::None,
                value: BigUint::dep_decode(input).unwrap(),
            },
            3 => CustomOption {
                id: CustomOptionId::Some,
                value: BigUint::dep_decode(input).unwrap(),
            },
            _ => return core::result::Result::Err(h.handle_error(DecodeError::from("invalid data"))),
        };

        let start_time = u64::dep_decode(input).unwrap();
        let deadline = u64::dep_decode(input).unwrap();
        let original_owner = ManagedAddress::dep_decode(input).unwrap();
        let current_bid = BigUint::dep_decode(input).unwrap();
        let current_winner = ManagedAddress::dep_decode(input).unwrap();
        let marketplace_cut_percentage = BigUint::dep_decode(input).unwrap();
        let creator_royalties_percentage = BigUint::dep_decode(input).unwrap();

        core::result::Result::Ok(
            Auction {
                auctioned_token_type,
                auctioned_token_nonce,
                nr_auctioned_tokens,
                auction_type,
                payment_token_type,
                payment_token_nonce,
                min_bid,
                max_bid,
                start_time,
                deadline,
                original_owner,
                current_bid,
                current_winner,
                marketplace_cut_percentage,
                creator_royalties_percentage
            }
        )
    }
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
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct BulkOffers<M: ManagedTypeApi> {
    pub offer: Offer<M>,
    pub offer_id: u64,
    pub nonce: u64,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub enum AuctionType {
    None,
    NftBid,
    Nft,
    SftAll,
    SftOnePerPayment,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug, Clone)]
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
