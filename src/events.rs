elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::auction::{Auction, AuctionType, OfferStatus, Offer};

#[allow(clippy::too_many_arguments)]
#[elrond_wasm::module]
pub trait EventsModule {
    fn emit_auction_token_event(self, auction_id: u64, auction: Auction<Self::Api>) {
        self.auction_token_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            &auction.nr_auctioned_tokens,
            &auction.original_owner,
            &auction.min_bid,
            &auction.max_bid.unwrap_or_else(|| BigUint::zero()),
            auction.start_time,
            auction.deadline,
            auction.payment_token_type,
            auction.payment_token_nonce,
            auction.auction_type,
            auction.creator_royalties_percentage,
        )
    }
    fn emit_offer_token_event(self, offer_id: u64, offer: Offer<Self::Api>) {
        self.offer_token_event(
            &offer.token_type,
            offer.token_nonce,
            &offer.quantity,
            offer.status,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &offer.price,
            offer.deadline,
            offer.timestamp,
            &offer.offer_owner,
            &offer.marketplace_cut_percentage,
            offer_id
        )
    }

    fn emit_withdraw_offer_event(self, offer_id: u64, offer: Offer<Self::Api>) {
        self.withdraw_offer_token_event(
            &offer.token_type,
            offer.token_nonce,
            &offer.quantity,
            offer.status,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &offer.price,
            offer.deadline,
            offer.timestamp,
            &offer.offer_owner,
            &offer.marketplace_cut_percentage,
            offer_id
        )
    }

    fn emit_accept_offer_event(self, offer_id: u64, offer: Offer<Self::Api>, seller: &ManagedAddress) {
        self.accept_offer_token_event(
            &offer.token_type,
            offer.token_nonce,
            &offer.quantity,
            offer.status,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &offer.price,
            offer.deadline,
            offer.timestamp,
            &offer.offer_owner,
            &offer.marketplace_cut_percentage,
            offer_id,
            seller,
        )
    }

    fn emit_bid_event(self, auction_id: u64, auction: Auction<Self::Api>) {
        self.bid_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            &auction.nr_auctioned_tokens,
            &auction.current_winner,
            &auction.current_bid,
            &auction.original_owner,
        );
    }

    fn emit_end_auction_event(self, auction_id: u64, auction: Auction<Self::Api>) {
        self.end_auction_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            &auction.nr_auctioned_tokens,
            &auction.current_winner,
            &auction.current_bid,
            &auction.original_owner,
        );
    }

    fn emit_buy_event(
        self,
        auction_id: u64,
        auction: Auction<Self::Api>,
        nr_bought_tokens: BigUint,
    ) {
        self.buy_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            &nr_bought_tokens,
            &auction.current_winner,
            &auction.min_bid,
            &auction.original_owner,
        );
    }

    fn emit_withdraw_event(self, auction_id: u64, auction: Auction<Self::Api>) {
        self.withdraw_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            &auction.nr_auctioned_tokens,
            &auction.original_owner,
        );
    }

    #[event("auction_token_event")]
    fn auction_token_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] auctioned_token_amount: &BigUint,
        #[indexed] seller: &ManagedAddress,
        #[indexed] min_bid: &BigUint,
        #[indexed] max_bid: &BigUint,
        #[indexed] start_time: u64,
        #[indexed] deadline: u64,
        #[indexed] accepted_payment_token: TokenIdentifier,
        #[indexed] accepted_payment_token_nonce: u64,
        #[indexed] auction_type: AuctionType,
        creator_royalties_percentage: BigUint, // between 0 and 10,000
    );

    #[event("offer_token_event")]
    fn offer_token_event(
        &self,
        #[indexed] token_type: &TokenIdentifier,
        #[indexed] token_nonce: u64,
        #[indexed] quantity: &BigUint,
        #[indexed] status: OfferStatus,
        #[indexed] payment_token_type: &TokenIdentifier,
        #[indexed] payment_token_nonce: u64,
        #[indexed] price: &BigUint,
        #[indexed] deadline: u64,
        #[indexed] timestamp: u64,
        #[indexed] offer_owner: &ManagedAddress,
        #[indexed] marketplace_cut_percentage: &BigUint,
        #[indexed] offer_id: u64,
    );

    #[event("withdraw_offer_token_event")]
    fn withdraw_offer_token_event(
        &self,
        #[indexed] token_type: &TokenIdentifier,
        #[indexed] token_nonce: u64,
        #[indexed] quantity: &BigUint,
        #[indexed] status: OfferStatus,
        #[indexed] payment_token_type: &TokenIdentifier,
        #[indexed] payment_token_nonce: u64,
        #[indexed] price: &BigUint,
        #[indexed] deadline: u64,
        #[indexed] timestamp: u64,
        #[indexed] offer_owner: &ManagedAddress,
        #[indexed] marketplace_cut_percentage: &BigUint,
        #[indexed] offer_id: u64,
    );

    #[event("accept_offer_token_event")]
    fn accept_offer_token_event(
        &self,
        #[indexed] token_type: &TokenIdentifier,
        #[indexed] token_nonce: u64,
        #[indexed] quantity: &BigUint,
        #[indexed] status: OfferStatus,
        #[indexed] payment_token_type: &TokenIdentifier,
        #[indexed] payment_token_nonce: u64,
        #[indexed] price: &BigUint,
        #[indexed] deadline: u64,
        #[indexed] timestamp: u64,
        #[indexed] offer_owner: &ManagedAddress,
        #[indexed] marketplace_cut_percentage: &BigUint,
        #[indexed] offer_id: u64,
        #[indexed] seller: &ManagedAddress,
    );

    #[event("bid_event")]
    fn bid_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] nr_auctioned_tokens: &BigUint,
        #[indexed] bidder: &ManagedAddress,
        #[indexed] bid_amount: &BigUint,
        #[indexed] seller: &ManagedAddress,
    );

    #[event("end_auction_event")]
    fn end_auction_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] nr_auctioned_tokens: &BigUint,
        #[indexed] auction_winner: &ManagedAddress,
        #[indexed] winning_bid_amount: &BigUint,
        #[indexed] auction_seller: &ManagedAddress,
    );

    #[event("buy_event")]
    fn buy_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] nr_bought_tokens: &BigUint,
        #[indexed] buyer: &ManagedAddress,
        #[indexed] bid_sft_amount: &BigUint,
        #[indexed] seller: &ManagedAddress,
    );

    #[event("withdraw_event")]
    fn withdraw_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] nr_auctioned_tokens: &BigUint,
        #[indexed] seller: &ManagedAddress,
    );
}
