elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::auction::GlobalOffer;
use crate::events;
use crate::helpers;
use crate::views;
use crate::{storage, NFT_AMOUNT, PERCENTAGE_TOTAL};

use super::auction::{AuctionType, Offer, OfferStatus};

#[elrond_wasm::module]
pub trait CustomOffersModule:
    storage::StorageModule + helpers::HelpersModule + events::EventsModule + views::ViewsModule
{
    #[payable("*")]
    #[endpoint(acceptOffer)]
    fn accept_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        offer_id: u64,
    ) {
        require!(self.status().get(), "Global operation enabled!");
        let mut offer = self.try_get_offer(offer_id);
        let current_time = self.blockchain().get_block_timestamp();
        require!(
            current_time <= offer.deadline,
            "Cannot accept the offer after deadline!"
        );
        let seller = self.blockchain().get_caller();
        require!(offer.offer_owner != seller, "Cannot accept your own offer!");
        let token_auction_ids_instance =
            self.token_auction_ids(offer.token_type.clone(), offer.token_nonce.clone());
        let mut found_match = false;
        let mut auction_removed = 0;
        if token_auction_ids_instance.is_empty() || payment_token.is_esdt() {
            require!(
                payment_amount == offer.quantity,
                "The quantity sent is not matching the offer!"
            );
            require!(
                payment_token_nonce == offer.token_nonce,
                "The nonce used is not matching the offer!"
            );
            require!(
                payment_token == offer.token_type,
                "The token sent is not matching the offer!"
            );
            found_match = true;
        } else if token_auction_ids_instance.len() == 1 {
            require!(
                token_auction_ids_instance.len() == 1,
                "You cannot accept offers for SFTs with more than 1 supply listed!"
            );
            let mut iter = token_auction_ids_instance.iter();
            let auction_id = iter.next().unwrap();
            let auction = self.try_get_auction(auction_id);
            require!(
                auction.auction_type == AuctionType::Nft,
                "Cannot accept offers for auctions, just for listings with a fixed price!"
            );

            require!(
                offer.offer_owner != auction.original_owner,
                "Cannot accept your own offer!"
            );

            require!(
                seller == auction.original_owner,
                "Just the owner of the listed NFT can accept the offer!"
            );

            require!(
                auction.nr_auctioned_tokens == offer.quantity,
                "The quantity listed is not matching the offer!"
            );
            require!(
                auction.auctioned_token_nonce == offer.token_nonce,
                "The nonce used is not matching the offer!"
            );
            require!(
                auction.auctioned_token_type == offer.token_type,
                "The listed token is not matching the offer!"
            );

            auction_removed = auction_id;
            self.listings_by_wallet(auction.original_owner.clone())
                .remove(&auction_id);
            self.token_auction_ids(offer.token_type.clone(), offer.token_nonce)
                .remove(&auction_id);
            self.auction_by_id(auction_id).clear();
            self.listings().remove(&auction_id);
            self.token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                .update(|qt| *qt -= &offer.quantity);

            if self
                .token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                .get()
                == BigUint::from(0u32)
            {
                self.token_items_for_sale(offer.token_type.clone())
                    .remove(&offer.token_nonce);
                self.token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                    .clear();
            }
            if self.token_items_for_sale(offer.token_type.clone()).len() == 0 {
                self.collections_listed().remove(&offer.token_type.clone());
            }

            found_match = true;
        } else {
            for auction_id in token_auction_ids_instance.iter() {
                let (
                    auctioned_token_type,
                    auctioned_token_nonce,
                    nr_auctioned_tokens,
                    owner_auction,
                    auction_type,
                ) = match self.get_auctioned_token_and_owner(auction_id) {
                    OptionalValue::Some(arg) => arg.into_tuple(),
                    OptionalValue::None => {
                        elrond_wasm::sc_panic!("The auction should have values!")
                    }
                };
                if offer.token_type == auctioned_token_type
                    && offer.token_nonce == auctioned_token_nonce
                    && offer.quantity == nr_auctioned_tokens
                    && seller == owner_auction
                    && (auction_type == AuctionType::Nft || auction_type == AuctionType::SftAll)
                {
                    auction_removed = auction_id;
                    self.listings_by_wallet(owner_auction).remove(&auction_id);
                    self.token_auction_ids(offer.token_type.clone(), offer.token_nonce)
                        .remove(&auction_id);
                    self.auction_by_id(auction_id).clear();
                    self.listings().remove(&auction_id);
                    self.token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                        .update(|qt| *qt -= &offer.quantity);

                    if self
                        .token_items_quantity_for_sale(offer.token_type.clone(), offer.token_nonce)
                        .get()
                        == BigUint::from(0u32)
                    {
                        self.token_items_for_sale(offer.token_type.clone())
                            .remove(&offer.token_nonce);
                        self.token_items_quantity_for_sale(
                            offer.token_type.clone(),
                            offer.token_nonce,
                        )
                        .clear();
                    }
                    found_match = true;
                    break;
                }
            }
        }

        require!(found_match, "No offer found for your accept!");
        offer.status = OfferStatus::Accepted;
        let nft_info = self.get_nft_info(&offer.token_type, offer.token_nonce);
        let creator_royalties_percentage = nft_info.royalties;
        require!(
            &offer.marketplace_cut_percentage + &creator_royalties_percentage < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );
        if !self.reward_ticker().is_empty() {
            if self
                .special_reward_amount(offer.token_type.clone())
                .is_empty()
            {
                if self.reward_balance().get().gt(&BigUint::from(0u32))
                    && self
                        .reward_balance()
                        .get()
                        .ge(&self.reward_amount().get().mul(2u32))
                {
                    self.transfer_or_save_payment(
                        &offer.offer_owner,
                        &self.reward_ticker().get(),
                        0u64,
                        &self.reward_amount().get(),
                        b"Trust Market rewards!",
                    );

                    self.transfer_or_save_payment(
                        &seller,
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
                    && self.reward_balance().get().ge(&self
                        .special_reward_amount(offer.token_type.clone())
                        .get()
                        .mul(2u32))
                {
                    self.transfer_or_save_payment(
                        &offer.offer_owner,
                        &self.reward_ticker().get(),
                        0u64,
                        &self.special_reward_amount(offer.token_type.clone()).get(),
                        b"Trust Market rewards!",
                    );

                    self.transfer_or_save_payment(
                        &seller,
                        &self.reward_ticker().get(),
                        0u64,
                        &self.special_reward_amount(offer.token_type.clone()).get(),
                        b"Trust Market rewards!",
                    );

                    self.reward_balance().update(|qt| {
                        *qt -= self
                            .special_reward_amount(offer.token_type.clone())
                            .get()
                            .mul(2u32)
                    });
                }
            }
        }
        self.transfer_or_save_payment(
            &offer.offer_owner,
            &offer.token_type,
            offer.token_nonce,
            &offer.quantity,
            b"Trust Market sent the bought token!",
        );

        let bid_split_amounts =
            self.calculate_offer_bid_split(&offer, &creator_royalties_percentage);

        let owner = self.blockchain().get_owner_address();
        self.transfer_or_save_payment(
            &owner,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &bid_split_amounts.marketplace,
            b"Trust Market fees revenue!",
        );

        self.transfer_or_save_payment(
            &nft_info.creator,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &bid_split_amounts.creator,
            b"Trust Market royalties for your token!",
        );

        // send rest of the offer to original seller
        self.transfer_or_save_payment(
            &seller,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &bid_split_amounts.seller,
            b"Trust Market income!",
        );
        self.check_offer_sent(
            offer.offer_owner.clone(),
            offer.token_type.clone(),
            offer.token_nonce.clone(),
            offer.payment_token_type.clone(),
        )
        .clear();
        self.token_offers_ids(offer.token_type.clone(), offer.token_nonce.clone())
            .remove(&offer_id);
        self.offers_by_wallet(offer.offer_owner.clone())
            .remove(&offer_id);
        self.offer_by_id(offer_id).clear();
        self.offers().remove(&offer_id);

        self.emit_accept_offer_event(offer_id, offer, &seller, auction_removed);
    }

    #[payable("*")]
    #[endpoint(declineOffer)]
    fn decline_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        offer_id: u64,
    ) {
        require!(self.status().get(), "Global operation enabled!");
        let offer = self.try_get_offer(offer_id);
        let owner = self.blockchain().get_caller();

        let token_auction_ids_instance =
            self.token_auction_ids(offer.token_type.clone(), offer.token_nonce.clone());
        if token_auction_ids_instance.is_empty() {
            require!(
                payment_amount == offer.quantity,
                "The quantity sent is not matching the offer!"
            );
            require!(
                payment_token_nonce == offer.token_nonce,
                "The nonce used is not matching the offer!"
            );
            require!(
                payment_token == offer.token_type,
                "The token sent is not matching the offer!"
            );
            self.send().direct(
                &owner,
                &payment_token,
                payment_token_nonce,
                &payment_amount,
                &[],
            );
        } else {
            require!(
                token_auction_ids_instance.len() == 1,
                "You cannot decline offers for SFTs with more than 1 supply minted!"
            );
            require!(
                payment_token_nonce == 0
                    && payment_token.is_egld()
                    && payment_amount.eq(&BigUint::zero()),
                "You have to send 0 eGLD as payment to decline the offer!"
            );
            let mut iter = token_auction_ids_instance.iter();
            let auction_id = iter.next().unwrap();
            let auction = self.try_get_auction(auction_id);
            require!(
                auction.auction_type == AuctionType::Nft,
                "Cannot decline offers for auctions, just for listings with a fixed price!"
            );
            require!(
                owner == auction.original_owner,
                "Just the owner of the NFT can decline the offer!"
            );
        }
        self.internal_withdraw_offer(offer_id);
    }

    #[endpoint(withdrawOffer)]
    fn withdraw_offer(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let mut offer = self.try_get_offer(offer_id);
        let caller = self.blockchain().get_caller();

        require!(
            offer.offer_owner == caller,
            "Only the original owner can withdraw the offer!"
        );

        self.send().direct(
            &caller,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &offer.price,
            self.data_or_empty_if_sc(&caller, b"Trust Market withdraw offer!"),
        );

        self.token_offers_ids(offer.token_type.clone(), offer.token_nonce.clone())
            .remove(&offer_id);
        self.check_offer_sent(
            offer.offer_owner.clone(),
            offer.token_type.clone(),
            offer.token_nonce.clone(),
            offer.payment_token_type.clone(),
        )
        .clear();
        self.offers_by_wallet(offer.offer_owner.clone())
            .remove(&offer_id);
        self.offers().remove(&offer_id);
        self.offer_by_id(offer_id).clear();
        offer.status = OfferStatus::Withdraw;
        self.emit_withdraw_offer_event(offer_id, offer);
    }

    fn internal_withdraw_offer(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let mut offer = self.try_get_offer(offer_id);

        self.send().direct(
            &offer.offer_owner,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &offer.price,
            self.data_or_empty_if_sc(&offer.offer_owner, b"Trust Market withdraw offer!"),
        );

        self.token_offers_ids(offer.token_type.clone(), offer.token_nonce.clone())
            .remove(&offer_id);
        self.check_offer_sent(
            offer.offer_owner.clone(),
            offer.token_type.clone(),
            offer.token_nonce.clone(),
            offer.payment_token_type.clone(),
        )
        .clear();
        self.offers_by_wallet(offer.offer_owner.clone())
            .remove(&offer_id);
        self.offers().remove(&offer_id);
        self.offer_by_id(offer_id).clear();
        offer.status = OfferStatus::Withdraw;
        self.emit_withdraw_offer_event(offer_id, offer);
    }

    #[endpoint(cleanExpiredOffers)]
    fn clean_expired_offers(&self) -> i32 {
        let timestamp = self.blockchain().get_block_timestamp();
        let mut found = 0;
        for offer_id in self.offers().iter() {
            let offer = self.offer_by_id(offer_id);
            if !offer.is_empty() {
                if offer.get().deadline < timestamp {
                    found += 1;
                    self.internal_withdraw_offer(offer_id);
                }
                if found == 150 {
                    break;
                }
            } else {
                self.offers().remove(&offer_id);
            }
        }
        found
    }

    fn try_get_offer(&self, offer_id: u64) -> Offer<Self::Api> {
        require!(self.does_offer_exist(offer_id), "Offer does not exist!");
        self.offer_by_id(offer_id).get()
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(addBlackListWallet)]
    fn add_blacklist(&self, wallet: ManagedAddress) -> bool {
        self.blacklist_wallets().insert(wallet)
    }

    #[only_owner]
    #[endpoint(removeBlackListWallet)]
    fn remove_blacklist(&self, wallet: ManagedAddress) -> bool {
        self.blacklist_wallets().remove(&wallet)
    }

    #[payable("*")]
    #[endpoint(sendOffer)]
    fn send_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        nft_amount: BigUint,
        deadline: u64,
    ) -> u64 {
        require!(self.status().get(), "Global operation enabled!");

        require!(
            self.accepted_tokens().contains(&payment_token),
            "The payment token is not whitelisted!"
        );
        require!(
            nft_nonce > 0,
            "Only Semi-Fungible and Non-Fungible tokens can have offers"
        );
        require!(
            nft_amount == BigUint::from(NFT_AMOUNT),
            "The quantity has to be 1!"
        );

        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();
        require!(
            !self.blacklist_wallets().contains(&caller),
            "Your address was blacklisted, all your SCAM offers are lost!"
        );
        require!(
            !self
                .check_offer_sent(
                    caller.clone(),
                    nft_type.clone(),
                    nft_nonce,
                    payment_token.clone()
                )
                .get(),
            "You already sent an offer for this NFT with the same token!"
        );
        if !payment_token.is_egld() {
            require!(
                payment_token.is_valid_esdt_identifier(),
                "The payment token is not valid!"
            );
        }

        require!(
            nft_type.is_valid_esdt_identifier(),
            "The NFT token is not valid!"
        );

        require!(deadline > current_time, "Deadline can't be in the past!");

        let marketplace_cut_percentage = self.bid_cut_percentage().get();

        let offer_id = self.last_valid_offer_id().get() + 1;
        self.last_valid_offer_id().set(&offer_id);

        let offer = Offer {
            token_type: nft_type.clone(),
            token_nonce: nft_nonce.clone(),
            quantity: nft_amount.clone(),
            payment_token_type: payment_token.clone(),
            payment_token_nonce,
            status: OfferStatus::Pending,
            price: payment_amount,
            deadline,
            timestamp: current_time,
            offer_owner: caller.clone(),
            marketplace_cut_percentage,
        };
        // Map ID with Offer Struct
        self.offer_by_id(offer_id).set(&offer);
        self.token_offers_ids(nft_type.clone(), nft_nonce)
            .insert(offer_id);
        // Push ID to the offers list
        self.offers().insert(offer_id);
        // Add to the owner wallet the new Offer ID
        self.offers_by_wallet(offer.offer_owner.clone())
            .insert(offer_id.clone());
        self.check_offer_sent(
            caller.clone(),
            nft_type.clone(),
            nft_nonce,
            payment_token.clone(),
        )
        .set(&true);
        // Emit event for new offer
        self.emit_offer_token_event(offer_id, offer);

        offer_id
    }

    #[payable("EGLD")]
    #[endpoint(sendGlobalOffer)]
    fn send_global_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_nonce: u64,
        #[payment_amount] price: BigUint,
        collection: TokenIdentifier,
    ) -> u64 {
        require!(self.status().get(), "Global operation enabled!");

        require!(
            self.accepted_tokens().contains(&payment_token),
            "The payment token is not whitelisted!"
        );

        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();
        require!(
            !self.blacklist_wallets().contains(&caller),
            "Your address was blacklisted!"
        );
        let mut user_map = self.user_collection_global_offers(&caller, &collection);
        require!(
            user_map.len() <= 5,
            "You have a limit of 5 offers per collection!"
        );
        let offer_id = self.last_valid_global_offer_id().get() + 1;
        let offer = GlobalOffer {
            offer_id,
            collection: collection.clone(),
            quantity: BigUint::from(NFT_AMOUNT),
            payment_token,
            payment_nonce,
            price,
            timestamp: current_time,
            owner: caller.clone(),
        };
        self.last_valid_global_offer_id().set(&offer_id);

        self.collection_global_offers(&collection).insert(offer_id);
        self.user_global_offers(&caller).insert(offer_id);
        user_map.insert(offer_id);
        self.emit_send_global_offer_event(&offer);
        self.global_offer_ids().insert(offer_id);
        self.global_offer(offer_id).set(offer);
        offer_id
    }

    #[endpoint(withdrawGlobalOffer)]
    fn withdraw_global_offer(&self, offer_id: u64) -> u64 {
        require!(self.status().get(), "Global operation enabled!");
        let caller = self.blockchain().get_caller();
        let offer_map = self.global_offer(offer_id);
        require!(!offer_map.is_empty(), "This offer is already removed!");
        let mut user_map = self.user_global_offers(&caller);
        require!(
            user_map.contains(&offer_id),
            "You are not the owner of this offer!"
        );
        let offer = offer_map.get();
        require!(
            offer.owner.eq(&caller),
            "You are not the owner of this offer!"
        );
        user_map.swap_remove(&offer_id);
        self.user_collection_global_offers(&caller, &offer.collection)
            .swap_remove(&offer_id);
        self.collection_global_offers(&offer.collection)
            .swap_remove(&offer_id);
        self.global_offer_ids().swap_remove(&offer_id);
        offer_map.clear();
        self.emit_remove_global_offer_event(offer_id);
        self.transfer_or_save_payment(
            &offer.owner,
            &offer.payment_token,
            offer.payment_nonce,
            &offer.price,
            &[],
        );
        offer_id
    }

    #[payable("*")]
    #[endpoint(acceptGlobalOffer)]
    fn accept_global_offer(
        &self,
        #[payment_token] collection: TokenIdentifier,
        #[payment_nonce] c_nonce: u64,
        #[payment_amount] amount: BigUint,
        offer_id: u64,
        #[var_args] auction_id_opt: OptionalValue<u64>,
    ) -> u64 {
        require!(self.status().get(), "Global operation enabled!");
        let offer_map = self.global_offer(offer_id);
        require!(!offer_map.is_empty(), "This offer is already removed!");
        let seller = self.blockchain().get_caller();
        let offer = offer_map.get();
        let mut collection_nonce = c_nonce;
        let auction_id_option = auction_id_opt.into_option();
        if auction_id_option.is_some() {
            require!(collection.is_empty(), "You don't have to send anything");
            require!(amount.eq(&BigUint::zero()), "Amount has to be 0");
            let auction_id = auction_id_option.unwrap();
            let auction = self.try_get_auction(auction_id);
            require!(
                auction.auction_type == AuctionType::Nft,
                "Cannot accept offers for auctions, just for listings with a fixed price!"
            );

            require!(
                offer.owner != auction.original_owner,
                "Cannot accept your own offer!"
            );

            require!(
                seller == auction.original_owner,
                "Just the owner of the listed NFT can accept the offer!"
            );

            require!(
                auction.nr_auctioned_tokens == offer.quantity,
                "The quantity listed is not matching the offer!"
            );

            require!(
                auction.auctioned_token_type == offer.collection,
                "The listed token is not matching the offer!"
            );
            collection_nonce = auction.auctioned_token_nonce;
            self.listings_by_wallet(auction.original_owner.clone())
                .remove(&auction_id);
            self.token_auction_ids(
                auction.auctioned_token_type.clone(),
                auction.auctioned_token_nonce,
            )
            .remove(&auction_id);
            self.auction_by_id(auction_id).clear();
            self.listings().remove(&auction_id);
            self.token_items_quantity_for_sale(
                auction.auctioned_token_type.clone(),
                auction.auctioned_token_nonce,
            )
            .update(|qt| *qt -= &offer.quantity);

            if self
                .token_items_quantity_for_sale(
                    auction.auctioned_token_type.clone(),
                    auction.auctioned_token_nonce,
                )
                .get()
                == BigUint::from(0u32)
            {
                self.token_items_for_sale(auction.auctioned_token_type.clone())
                    .remove(&auction.auctioned_token_nonce);
                self.token_items_quantity_for_sale(
                    auction.auctioned_token_type.clone(),
                    auction.auctioned_token_nonce,
                )
                .clear();
            }
            if self
                .token_items_for_sale(auction.auctioned_token_type.clone())
                .len()
                == 0
            {
                self.collections_listed()
                    .remove(&auction.auctioned_token_type.clone());
            }
        } else {
            require!(collection_nonce > 0, "You can not accept it with ESDT!");
            require!(
                offer.collection.eq(&collection),
                "The collection sent is not the offer requested one!"
            );
            require!(
                offer.quantity.eq(&amount),
                "Your quantity is not matching the offer requested one!"
            );
        }
        self.user_collection_global_offers(&offer.owner, &offer.collection)
            .swap_remove(&offer.offer_id);
        self.collection_global_offers(&offer.collection)
            .swap_remove(&offer.offer_id);
        self.user_global_offers(&offer.owner)
            .swap_remove(&offer.offer_id);
        self.global_offer(offer.offer_id).clear();
        self.global_offer_ids().swap_remove(&offer.offer_id);

        let nft_info = self.get_nft_info(&offer.collection, collection_nonce);
        let payments = self.calculate_global_offer_split(&offer, &nft_info);

        self.transfer_or_save_payment(
            &self.blockchain().get_owner_address(),
            &offer.payment_token,
            offer.payment_nonce,
            &payments.marketplace,
            b"Trust Market fees revenue!",
        );

        self.transfer_or_save_payment(
            &nft_info.creator,
            &offer.payment_token,
            offer.payment_nonce,
            &payments.creator,
            b"Trust Market royalties for your token!",
        );

        // send rest of the offer to original seller
        self.transfer_or_save_payment(
            &seller,
            &offer.payment_token,
            offer.payment_nonce,
            &payments.seller,
            b"Trust Market income!",
        );

        self.transfer_or_save_payment(
            &offer.owner,
            &offer.collection,
            collection_nonce,
            &offer.quantity,
            b"Trust Market income!",
        );

        self.emit_accept_global_offer_event(
            &offer,
            &seller,
            collection_nonce,
            &offer.quantity,
            auction_id_option.unwrap_or(0u64),
        );
        offer_id
    }

    #[only_owner]
    #[endpoint(deleteOffersByWallet)]
    fn delete_user_offers(&self, user: ManagedAddress) {
        let offers_root = self.offers_by_wallet(user.clone());
        if offers_root.len() > 0 {
            for offer in offers_root.iter().take(80) {
                self.internal_withdraw_offer(offer);
            }
        }
    }

    #[view(doesOfferExist)]
    fn does_offer_exist(&self, offer_id: u64) -> bool {
        !self.offer_by_id(offer_id).is_empty()
    }
}
