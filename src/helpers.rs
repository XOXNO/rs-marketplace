elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use core::convert::TryInto;

use crate::{
    auction::{Auction, AuctionType, BidSplitAmounts, GlobalOffer, Offer},
    NFT_AMOUNT, PERCENTAGE_TOTAL,
};

#[elrond_wasm::module]
pub trait HelpersModule: crate::storage::StorageModule + crate::views::ViewsModule {
    fn transfer_or_save_payment(
        &self,
        to: &ManagedAddress,
        token_id: &TokenIdentifier,
        nonce: u64,
        amount: &BigUint,
        data: &'static [u8],
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
            self.send().direct(
                to,
                token_id,
                nonce,
                amount,
                self.data_or_empty_if_sc(to, data),
            );
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
            "Auction does not exist!"
        );
        self.auction_by_id(auction_id).get()
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
        let creator_royalties =
            self.calculate_cut_amount(&offer.price, &nft_info.royalties);
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

    fn distribute_tokens(&self, auction: &Auction<Self::Api>, opt_sft_amount: Option<&BigUint>) {
        let nft_type = &auction.auctioned_token_type;
        let nft_nonce = auction.auctioned_token_nonce;
        if !auction.current_winner.is_zero() {
            let nft_info = self.get_nft_info(nft_type, nft_nonce);
            let token_id = &auction.payment_token_type;
            let nonce = auction.payment_token_nonce;
            let bid_split_amounts = self.calculate_winning_bid_split(auction);

            // send part as cut for contract owner
            let owner = self.blockchain().get_owner_address();
            self.transfer_or_save_payment(
                &owner,
                token_id,
                nonce,
                &bid_split_amounts.marketplace,
                b"Trust Market fees revenue!",
            );

            self.transfer_or_save_payment(
                &nft_info.creator,
                token_id,
                nonce,
                &bid_split_amounts.creator,
                b"Trust Market royalties for your token!",
            );

            // send rest of the bid to original owner
            self.transfer_or_save_payment(
                &auction.original_owner,
                token_id,
                nonce,
                &bid_split_amounts.seller,
                b"Trust Market income!",
            );
            if !self.reward_ticker().is_empty() {
                if self.special_reward_amount(nft_type.clone()).is_empty() {
                    if self.reward_balance().get().gt(&BigUint::from(0u32))
                        && self
                            .reward_balance()
                            .get()
                            .ge(&self.reward_amount().get().mul(2u32))
                    {
                        self.transfer_or_save_payment(
                            &auction.original_owner,
                            &self.reward_ticker().get(),
                            0u64,
                            &self.reward_amount().get(),
                            b"Trust Market rewards!",
                        );

                        self.transfer_or_save_payment(
                            &auction.current_winner,
                            &self.reward_ticker().get(),
                            0u64,
                            &self.reward_amount().get(),
                            b"Trust Market rewards!",
                        );
                        self.reward_balance()
                            .update(|qt| *qt -= self.reward_amount().get().mul(2u32));
                    }
                } else {
                    if self.reward_balance().get().gt(&BigUint::from(0u32))
                        && self
                            .reward_balance()
                            .get()
                            .ge(&self.special_reward_amount(nft_type.clone()).get().mul(2u32))
                    {
                        self.transfer_or_save_payment(
                            &auction.original_owner,
                            &self.reward_ticker().get(),
                            0u64,
                            &self.special_reward_amount(nft_type.clone()).get(),
                            b"Trust Market rewards!",
                        );

                        self.transfer_or_save_payment(
                            &auction.current_winner,
                            &self.reward_ticker().get(),
                            0u64,
                            &self.special_reward_amount(nft_type.clone()).get(),
                            b"Trust Market rewards!",
                        );

                        self.reward_balance().update(|qt| {
                            *qt -= self.special_reward_amount(nft_type.clone()).get().mul(2u32)
                        });
                    }
                }
            }
            // send NFT to auction winner
            let nft_amount = BigUint::from(NFT_AMOUNT);
            let nft_amount_to_send = match auction.auction_type {
                AuctionType::Nft => &nft_amount,
                AuctionType::NftBid => &nft_amount,
                AuctionType::SftOnePerPayment => match opt_sft_amount {
                    Some(amt) => amt,
                    None => &nft_amount,
                },
                _ => &auction.nr_auctioned_tokens,
            };
            self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .update(|qt| *qt -= nft_amount_to_send.clone());

            if self
                .token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .get()
                .eq(&BigUint::from(0u32))
            {
                self.token_items_for_sale(nft_type.clone())
                    .remove(&nft_nonce);
                self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                    .clear();
            }
            if self.token_items_for_sale(nft_type.clone()).len() == 0 {
                self.collections_listed().remove(&nft_type);
            }

            self.transfer_or_save_payment(
                &auction.current_winner,
                nft_type,
                nft_nonce,
                nft_amount_to_send,
                b"Trust Market sent the bought token!",
            );
        } else {
            // return to original owner

            self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .update(|qt| *qt -= &auction.nr_auctioned_tokens);
            let quantity_token = self
                .token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .get();
            if quantity_token.eq(&BigUint::from(0u32)) {
                self.token_items_for_sale(nft_type.clone())
                    .remove(&nft_nonce);
                self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                    .clear();
            }

            if self.token_items_for_sale(nft_type.clone()).len() == 0 {
                self.collections_listed().remove(&nft_type);
            }

            self.transfer_or_save_payment(
                &auction.original_owner,
                nft_type,
                nft_nonce,
                &auction.nr_auctioned_tokens,
                b"Trust Market returned your token!",
            );
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
