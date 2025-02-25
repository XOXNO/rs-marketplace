multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use core::convert::TryInto;

use crate::{
    auction::{AttributesIns, Auction, CollectionFeeConfig, FeesDistribution, GlobalOffer, Offer},
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
        if self.blockchain().is_smart_contract(to) {
            if !self.whitelisted_contracts().contains(&to) {
                if amount == &0 {
                    return;
                }
                self.claimable_tokens(to).insert(token_id.clone());
                self.claimable_token_nonces(to, token_id).insert(nonce);
                self.claimable_amount(to, token_id, nonce)
                    .update(|amt| *amt += amount);
                return;
            }
        }
        self.send().direct_non_zero(to, token_id, nonce, amount);
    }

    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        let mut data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        );

        if nft_type == &TokenIdentifier::from_esdt_bytes(b"INS-dd5a76") {
            let attributes = data.decode_attributes::<AttributesIns<Self::Api>>();
            data.creator = attributes.creator;
        }

        data
    }

    fn try_get_auction(&self, auction_id: u64) -> Auction<Self::Api> {
        let map = self.auction_by_id(auction_id);
        require!(!map.is_empty(), "Auction {} does not exist!", auction_id);
        map.get()
    }

    fn try_get_global_offer(&self, offer_id: u64) -> GlobalOffer<Self::Api> {
        let map = self.global_offer(offer_id);
        require!(!map.is_empty(), "Global Offer {} does not exist!", offer_id);
        map.get()
    }

    fn try_get_offer(&self, offer_id: u64) -> Offer<Self::Api> {
        let map = self.offer_by_id(offer_id);
        require!(!map.is_empty(), "Offer {} does not exist!", offer_id);
        map.get()
    }

    fn try_set_bid_cut_percentage(&self, new_cut_percentage: u64) {
        require!(
            new_cut_percentage > 0 && new_cut_percentage < PERCENTAGE_TOTAL,
            "Invalid percentage value, should be between 0 and 10,000"
        );

        self.bid_cut_percentage()
            .set(&BigUint::from(new_cut_percentage));
    }

    fn calculate_cut_amount(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / PERCENTAGE_TOTAL
    }

    fn get_collection_config(
        &self,
        collection: &TokenIdentifier,
    ) -> Option<CollectionFeeConfig<Self::Api>> {
        let map = self.collection_config(collection);
        return match map.is_empty() {
            true => None,
            false => Some(map.get()),
        };
    }

    fn calculate_amount_split(
        &self,
        price: &BigUint,
        royalties: &BigUint,
        config: Option<CollectionFeeConfig<Self::Api>>,
    ) -> FeesDistribution<Self::Api> {
        let fees = self.bid_cut_percentage().get();
        let mut eligible_royalties = royalties.clone();
        let mut extra_amount = BigUint::zero();
        let mut reverse_royalties = false;
        let mut reverse_cut_fees = false;
        let mut extra_fee = BigUint::zero();
        let mut extra_address = ManagedAddress::zero();

        let _ = match config {
            Some(config) => {
                extra_fee = config.extra_fees.amount;
                extra_address = config.extra_fees.address;
                reverse_royalties = config.reverse_royalties;
                reverse_cut_fees = config.reverse_cut_fees;
                if config.custom_royalties {
                    if config.max_royalties < eligible_royalties {
                        eligible_royalties = config.max_royalties;
                    } else if config.min_royalties > eligible_royalties {
                        eligible_royalties = config.min_royalties;
                    }
                }
            }
            None => {}
        };

        require!(
            &fees + &eligible_royalties + &extra_fee < PERCENTAGE_TOTAL,
            "Fees exceed 100%"
        );
        let creator_royalties = self.calculate_cut_amount(price, &eligible_royalties);
        let marketplace_fees = self.calculate_cut_amount(price, &fees);
        let mut seller_amount_to_send = price.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &marketplace_fees;
        if extra_fee > BigUint::zero() && extra_address != ManagedAddress::zero() {
            extra_amount = self.calculate_cut_amount(&price, &extra_fee);
            seller_amount_to_send -= &extra_amount;
        }

        FeesDistribution {
            creator: creator_royalties,
            marketplace: marketplace_fees,
            extra: extra_amount,
            seller: seller_amount_to_send,
            extra_address,
            reverse_royalties,
            reverse_cut_fees,
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

    fn require_admin(&self, extra_admin: Option<ManagedAddress>) {
        let signer: ManagedAddress = self.signer().get();
        let caller = self.blockchain().get_caller();
        let sc_owner = self.blockchain().get_owner_address();
        if extra_admin.is_some() {
            require!(
                caller.eq(&sc_owner) || caller.eq(&signer) || caller.eq(&extra_admin.unwrap()),
                "You are not an admin!"
            );
        } else {
            require!(
                caller.eq(&sc_owner) || caller.eq(&signer),
                "You are not an admin!"
            );
        }
    }

    fn require_enabled(&self) {
        require!(self.status().get(), "Global operation enabled!");
    }
}
