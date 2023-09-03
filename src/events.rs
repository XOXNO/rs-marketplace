multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::auction::GlobalOffer;

use super::auction::{Auction, AuctionType, Offer, OfferStatus};

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_change_listing_event(
        self,
        auction_id: u64,
        auction: &Auction<Self::Api>,
        new_amount: &BigUint,
    ) {
        self.change_listing_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            &auction.original_owner,
            &auction.min_bid,
            new_amount,
            &auction.payment_token_type,
            auction.payment_token_nonce,
            auction.deadline,
        )
    }

    fn emit_out_bid_event(
        self,
        auction_id: u64,
        auction: &Auction<Self::Api>,
        bidder: &ManagedAddress,
        new_amount: &BigUint,
    ) {
        self.out_bid_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            &auction.current_winner,
            bidder,
            &auction.current_bid,
            new_amount,
            &auction.payment_token_type,
            auction.payment_token_nonce,
        )
    }

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
            offer_id,
        )
    }

    fn emit_withdraw_offer_event(self, offer_id: u64, offer: &Offer<Self::Api>) {
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
            offer_id,
        )
    }

    fn emit_accept_offer_event(
        self,
        offer_id: u64,
        offer: &Offer<Self::Api>,
        seller: &ManagedAddress,
        auction_removed: u64,
    ) {
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
            auction_removed,
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
            &auction.payment_token_type,
            auction.payment_token_nonce,
        );
    }

    fn emit_end_auction_event(self, auction_id: u64, auction: &Auction<Self::Api>) {
        self.end_auction_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            &auction.nr_auctioned_tokens,
            &auction.current_winner,
            &auction.current_bid,
            &auction.original_owner,
            &auction.payment_token_type,
            auction.payment_token_nonce,
        );
    }

    fn emit_buy_event(
        self,
        auction_id: u64,
        auction: &Auction<Self::Api>,
        nr_bought_tokens: &BigUint,
        current_time: u64,
        message: OptionalValue<ManagedBuffer>,
        buy_by: OptionalValue<ManagedAddress>,
    ) {
        self.buy_event(
            &auction.auctioned_token_type,
            auction.auctioned_token_nonce,
            auction_id,
            nr_bought_tokens,
            &auction.current_winner,
            &auction.min_bid,
            &auction.original_owner,
            &auction.payment_token_type,
            auction.payment_token_nonce,
            current_time,
            message.into_option().unwrap_or(ManagedBuffer::new()),
            buy_by.into_option().unwrap_or(ManagedAddress::default()),
            &auction.nr_auctioned_tokens,
        )
    }

    fn emit_withdraw_event(self, auction_id: u64, auction: &Auction<Self::Api>) {
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
        #[indexed] accepted_payment_token: EgldOrEsdtTokenIdentifier,
        #[indexed] accepted_payment_token_nonce: u64,
        #[indexed] auction_type: AuctionType,
        #[indexed] creator_royalties_percentage: BigUint,
    );

    #[event("offer_token_event")]
    fn offer_token_event(
        &self,
        #[indexed] token_type: &TokenIdentifier,
        #[indexed] token_nonce: u64,
        #[indexed] quantity: &BigUint,
        #[indexed] status: OfferStatus,
        #[indexed] payment_token_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] payment_token_nonce: u64,
        #[indexed] price: &BigUint,
        #[indexed] deadline: u64,
        #[indexed] timestamp: u64,
        #[indexed] offer_owner: &ManagedAddress,
        #[indexed] marketplace_cut_percentage: &BigUint,
        #[indexed] offer_id: u64,
    );

    fn emit_send_global_offer_event(self, offer: &GlobalOffer<Self::Api>) {
        self.send_global_offer_event(offer);
    }

    #[event("send_global_offer")]
    fn send_global_offer_event(&self, #[indexed] offer: &GlobalOffer<Self::Api>);

    fn emit_remove_global_offer_event(self, offer_id: u64, collection: &TokenIdentifier) {
        self.remove_global_offer_event(offer_id, collection);
    }

    #[event("remove_global_offer")]
    fn remove_global_offer_event(
        &self,
        #[indexed] offer_id: u64,
        #[indexed] collection: &TokenIdentifier,
    );

    fn emit_accept_global_offer_event(
        self,
        offer: &GlobalOffer<Self::Api>,
        seller: &ManagedAddress,
        nonce: u64,
        amount: &BigUint,
        auction_id: u64,
    ) {
        self.accept_global_offer_event(offer, seller, nonce, amount, auction_id);
    }

    #[event("accept_global_offer")]
    fn accept_global_offer_event(
        &self,
        #[indexed] offer: &GlobalOffer<Self::Api>,
        #[indexed] seller: &ManagedAddress,
        #[indexed] nonce: u64,
        #[indexed] amount: &BigUint,
        #[indexed] auction_id: u64,
    );

    #[event("withdraw_offer_token_event")]
    fn withdraw_offer_token_event(
        &self,
        #[indexed] token_type: &TokenIdentifier,
        #[indexed] token_nonce: u64,
        #[indexed] quantity: &BigUint,
        #[indexed] status: OfferStatus,
        #[indexed] payment_token_type: &EgldOrEsdtTokenIdentifier,
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
        #[indexed] payment_token_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] payment_token_nonce: u64,
        #[indexed] price: &BigUint,
        #[indexed] deadline: u64,
        #[indexed] timestamp: u64,
        #[indexed] offer_owner: &ManagedAddress,
        #[indexed] marketplace_cut_percentage: &BigUint,
        #[indexed] offer_id: u64,
        #[indexed] seller: &ManagedAddress,
        #[indexed] auction_removed: u64,
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
        #[indexed] token_payment_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] token_payment_nonce: u64,
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
        #[indexed] token_payment_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] token_payment_nonce: u64,
    );

    #[event("change_listing_event")]
    fn change_listing_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] owner: &ManagedAddress,
        #[indexed] old_price: &BigUint,
        #[indexed] new_price: &BigUint,
        #[indexed] payment_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] payment_nonce: u64,
        #[indexed] deadline: u64,
    );

    #[event("out_bid_event")]
    fn out_bid_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] old_bidder: &ManagedAddress,
        #[indexed] new_bidder: &ManagedAddress,
        #[indexed] refund_amount: &BigUint,
        #[indexed] new_amount: &BigUint,
        #[indexed] refund_payment_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] refund_payment_nonce: u64,
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
        #[indexed] accepted_payment_token: &EgldOrEsdtTokenIdentifier,
        #[indexed] accepted_payment_token_nonce: u64,
        #[indexed] timestamp: u64,
        #[indexed] message: ManagedBuffer,
        #[indexed] buy_by: ManagedAddress,
        #[indexed] nr_auctioned_tokens: &BigUint,
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

    #[event("user_deposit")]
    fn emit_deposit_balance(
        &self,
        #[indexed] owner: &ManagedAddress,
        #[indexed] payment: &EgldOrEsdtTokenPayment,
    );
}
