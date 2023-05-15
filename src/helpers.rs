multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use core::convert::TryInto;

use crate::{
    auction::{Auction, BidSplitAmounts, GlobalOffer, Offer},
    PERCENTAGE_TOTAL,
};

#[multiversx_sc::module]
pub trait HelpersModule:
    crate::storage::StorageModule + crate::views::ViewsModule + crate::events::EventsModule
{
    fn transfer_or_save_payment(
        &self,
        to: &ManagedAddress,
        token_id: &EgldOrEsdtTokenIdentifier,
        nonce: u64,
        amount: &BigUint,
    ) {
        if amount == &0 {
            return;
        }
        if self.blockchain().is_smart_contract(to) && !self.whitelisted_contracts().contains(&to) {
            self.claimable_tokens(to).insert(token_id.clone());
            self.claimable_token_nonces(to, token_id).insert(nonce);
            self.claimable_amount(to, token_id, nonce)
                .update(|amt| *amt += amount);
        } else {
            self.send().direct(to, token_id, nonce, amount);
        }
    }

    fn data_or_empty_if_sc(&self, dest: &ManagedAddress, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(dest) {
            &[]
        } else {
            data
        }
    }

    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }

    fn try_set_bid_cut_percentage(&self, new_cut_percentage: u64) {
        require!(
            new_cut_percentage > 0 && new_cut_percentage < PERCENTAGE_TOTAL,
            "Invalid percentage value, should be between 0 and 10,000"
        );

        self.bid_cut_percentage()
            .set(&BigUint::from(new_cut_percentage));
    }

    fn try_get_auction(&self, auction_id: u64) -> Auction<Self::Api> {
        require!(
            self.does_auction_exist(auction_id),
            "Auction {} does not exist!", auction_id
        );
        self.auction_by_id(auction_id).get()
    }

    fn try_get_global_offer(&self, offer_id: u64) -> GlobalOffer<Self::Api> {
        require!(
            self.does_global_offer_exist(offer_id),
            "Global Offer {} does not exist!", offer_id
        );
        self.global_offer(offer_id).get()
    }

    fn try_get_offer(&self, offer_id: u64) -> Offer<Self::Api> {
        require!(
            self.does_offer_exist(offer_id),
            "Offer {} does not exist!", offer_id
        );
        self.offer_by_id(offer_id).get()
    }

    fn calculate_cut_amount(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / PERCENTAGE_TOTAL
    }

    fn calculate_winning_bid_split(
        &self,
        auction: &Auction<Self::Api>,
    ) -> BidSplitAmounts<Self::Api> {
        let creator_royalties =
            self.calculate_cut_amount(&auction.current_bid, &auction.creator_royalties_percentage);
        let bid_cut_amount =
            self.calculate_cut_amount(&auction.current_bid, &auction.marketplace_cut_percentage);
        let mut seller_amount_to_send = auction.current_bid.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &bid_cut_amount;

        BidSplitAmounts {
            creator: creator_royalties,
            marketplace: bid_cut_amount,
            seller: seller_amount_to_send,
        }
    }

    fn calculate_offer_bid_split(
        &self,
        offer: &Offer<Self::Api>,
        creator_royalties_percentage: &BigUint,
    ) -> BidSplitAmounts<Self::Api> {
        let creator_royalties =
            self.calculate_cut_amount(&offer.price, &creator_royalties_percentage);
        let bid_cut_amount =
            self.calculate_cut_amount(&offer.price, &offer.marketplace_cut_percentage);
        let mut seller_amount_to_send = offer.price.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &bid_cut_amount;

        BidSplitAmounts {
            creator: creator_royalties,
            marketplace: bid_cut_amount,
            seller: seller_amount_to_send,
        }
    }

    fn calculate_global_offer_split(
        &self,
        offer: &GlobalOffer<Self::Api>,
        nft_info: &EsdtTokenData<Self::Api>,
    ) -> BidSplitAmounts<Self::Api> {
        let cut_fee = self.bid_cut_percentage().get();
        require!(
            &cut_fee + &nft_info.royalties < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );
        let creator_royalties = self.calculate_cut_amount(&offer.price, &nft_info.royalties);
        let bid_cut_amount = self.calculate_cut_amount(&offer.price, &cut_fee);
        let mut seller_amount_to_send = offer.price.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &bid_cut_amount;

        BidSplitAmounts {
            creator: creator_royalties,
            marketplace: bid_cut_amount,
            seller: seller_amount_to_send,
        }
    }

    fn decimal_to_ascii(&self, mut number: u32) -> ManagedBuffer {
        const MAX_NUMBER_CHARACTERS: usize = 10;
        const ZERO_ASCII: u8 = b'0';

        let mut as_ascii = [0u8; MAX_NUMBER_CHARACTERS];
        let mut nr_chars = 0;

        loop {
            let reminder: u8 = (number % 10).try_into().unwrap();
            number /= 10;

            as_ascii[nr_chars] = ZERO_ASCII + reminder;
            nr_chars += 1;

            if number == 0 {
                break;
            }
        }

        let slice = &mut as_ascii[..nr_chars];
        slice.reverse();

        ManagedBuffer::new_from_bytes(slice)
    }
}
