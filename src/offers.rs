elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use core::convert::TryInto;

use crate::auction::GlobalOffer;
use crate::common;
use crate::events;
use crate::helpers;
use crate::views;
use crate::wrapping;
use crate::{storage, NFT_AMOUNT, PERCENTAGE_TOTAL};
use elrond_wasm::api::ED25519_SIGNATURE_BYTE_LEN;

use super::auction::{AuctionType, Offer, OfferStatus};

const MAX_DATA_LEN: usize = 15000;

pub type Signature<M> = ManagedByteArray<M, ED25519_SIGNATURE_BYTE_LEN>;
#[elrond_wasm::module]
pub trait CustomOffersModule:
    storage::StorageModule
    + helpers::HelpersModule
    + events::EventsModule
    + views::ViewsModule
    + common::CommonModule
    + wrapping::WrappingModule
{
    #[payable("*")]
    #[endpoint(acceptOffer)]
    fn accept_offer(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let (payment_token, payment_token_nonce, payment_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();
        let mut offer = self.try_get_offer(offer_id);
        let current_time = self.blockchain().get_block_timestamp();
        require!(
            current_time <= offer.deadline,
            "Cannot accept the offer after deadline!"
        );
        let seller = self.blockchain().get_caller();
        require!(offer.offer_owner != seller, "Cannot accept your own offer!");
        let token_auction_ids_instance =
            self.token_auction_ids(&offer.token_type, offer.token_nonce);
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
            let mut iter = token_auction_ids_instance.iter();
            let auction_id = iter.next().unwrap();
            let auction = self.try_get_auction(auction_id);
            self.common_offer_auction_check(&offer, &auction);

            require!(
                seller == auction.original_owner,
                "Just the owner of the listed NFT can accept the offer!"
            );

            self.update_or_remove_items_quantity(&auction, &auction.nr_auctioned_tokens);
            self.remove_auction_common(auction_id, &auction);
            auction_removed = auction_id;
            found_match = true;
        } else {
            for auction_id in token_auction_ids_instance.iter() {
                let auction = match self.get_auctioned_token_and_owner(auction_id) {
                    OptionalValue::Some(auc) => auc,
                    OptionalValue::None => {
                        sc_panic!("The auction should have values!")
                    }
                };
                if offer.token_type == auction.auctioned_token_type
                    && offer.token_nonce == auction.auctioned_token_nonce
                    && offer.quantity == auction.nr_auctioned_tokens
                    && seller == auction.original_owner
                    && (auction.auction_type == AuctionType::Nft
                        || auction.auction_type == AuctionType::SftAll)
                {
                    self.update_or_remove_items_quantity(&auction, &auction.nr_auctioned_tokens);
                    self.remove_auction_common(auction_id, &auction);
                    auction_removed = auction_id;
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

        self.distribute_tokens_common(
            &EgldOrEsdtTokenIdentifier::esdt(offer.token_type.clone()),
            offer.token_nonce,
            &offer.quantity,
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &nft_info.creator,
            &seller,
            &offer.offer_owner,
            &self.calculate_offer_bid_split(&offer, &creator_royalties_percentage),
            false,
        );
        self.common_offer_remove(offer_id, &offer);
        self.emit_accept_offer_event(offer_id, offer, &seller, auction_removed);
    }

    #[payable("*")]
    #[endpoint(declineOffer)]
    fn decline_offer(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let (payment_token, payment_token_nonce, payment_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();
        let offer = self.try_get_offer(offer_id);
        let owner = self.blockchain().get_caller();

        let token_auction_ids_instance =
            self.token_auction_ids(&offer.token_type, offer.token_nonce);
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
            // return the NFT sent for confirmation
            self.send()
                .direct(&owner, &payment_token, payment_token_nonce, &payment_amount);
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
                owner == auction.original_owner,
                "Just the owner of the NFT can decline the offer!"
            );
            self.common_offer_auction_check(&offer, &auction);
        }
        self.common_withdraw_offer(offer_id, &offer);
    }

    #[endpoint(withdrawOffer)]
    fn withdraw_offer(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let offer = self.try_get_offer(offer_id);
        let caller = self.blockchain().get_caller();

        require!(
            offer.offer_owner == caller,
            "Only the original owner can withdraw the offer!"
        );

        self.common_withdraw_offer(offer_id, &offer);
        self.emit_withdraw_offer_event(offer_id, &offer);
    }

    #[payable("EGLD")]
    #[endpoint(sendOffer)]
    fn send_offer(
        &self,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        nft_amount: BigUint,
        deadline: u64,
    ) -> u64 {
        require!(self.status().get(), "Global operation enabled!");
        let (payment_token, payment_token_nonce, payment_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();
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
                .check_offer_sent(&caller, &nft_type, nft_nonce, &payment_token)
                .get(),
            "You already sent an offer for this NFT with the same token!"
        );

        require!(payment_token.is_egld(), "The payment token is not valid!");

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
            token_nonce: nft_nonce,
            quantity: nft_amount,
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
        self.token_offers_ids(&nft_type, nft_nonce).insert(offer_id);
        // Push ID to the offers list
        self.offers().insert(offer_id);
        // Add to the owner wallet the new Offer ID
        self.offers_by_wallet(&offer.offer_owner).insert(offer_id);
        self.check_offer_sent(&caller, &nft_type, nft_nonce, &payment_token)
            .set(&true);
        // Emit event for new offer
        self.emit_offer_token_event(offer_id, offer);

        offer_id
    }

    #[payable("EGLD")]
    #[endpoint(sendGlobalOffer)]
    fn send_global_offer(
        &self,
        #[payment_token] payment_token: EgldOrEsdtTokenIdentifier,
        #[payment_nonce] payment_nonce: u64,
        #[payment_amount] price: BigUint,
        collection: TokenIdentifier,
        attributes: OptionalValue<ManagedBuffer>,
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
            attributes: attributes.into_option(),
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
    fn withdraw_global_offer(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let caller = self.blockchain().get_caller();
        let offer = self.try_get_global_offer(offer_id);
        require!(
            offer.owner.eq(&caller),
            "You are not the owner of this offer!"
        );
        self.common_global_offer_remove(&offer, true);
        self.emit_remove_global_offer_event(offer_id);
    }

    #[payable("*")]
    #[endpoint(acceptGlobalOffer)]
    fn accept_global_offer(
        &self,
        offer_id: u64,
        auction_id_opt: OptionalValue<u64>,
        signature: OptionalValue<Signature<Self::Api>>,
    ) -> u64 {
        require!(self.status().get(), "Global operation enabled!");
        let (collection, c_nonce, amount) = self.call_value().egld_or_single_esdt().into_tuple();
        let offer_map = self.global_offer(offer_id);
        require!(!offer_map.is_empty(), "This offer is already removed!");
        let seller = self.blockchain().get_caller();
        let offer = offer_map.get();
        let mut collection_nonce = c_nonce;
        let auction_id_option = auction_id_opt.into_option();
        if auction_id_option.is_some() && auction_id_option.unwrap() > 0 {
            require!(collection.is_egld(), "You don't have to send anything");
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
            self.update_or_remove_items_quantity(&auction, &auction.nr_auctioned_tokens);
            self.remove_auction_common(auction_id, &auction);
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
        if offer.attributes.is_some() {
            let sign = signature.into_option();
            require!(sign.is_some(), "Signature required!");
            let mut data = ManagedBuffer::new();
            data.append(seller.as_managed_buffer());
            data.append(offer.collection.as_managed_buffer());
            data.append(&self.decimal_to_ascii(collection_nonce.try_into().unwrap()));
            data.append(&self.decimal_to_ascii(offer.offer_id.try_into().unwrap()));
            data.append(&offer.attributes.as_ref().unwrap());

            let signer: ManagedAddress = self.signer().get();
            let valid_signature = self.crypto().verify_ed25519_legacy_managed::<MAX_DATA_LEN>(
                signer.as_managed_byte_array(),
                &data,
                &sign.unwrap(),
            );
            require!(valid_signature, "Invalid signature");
        }

        self.common_global_offer_remove(&offer, false);
        let nft_info = self.get_nft_info(&offer.collection, collection_nonce);

        self.distribute_tokens_common(
            &EgldOrEsdtTokenIdentifier::esdt(offer.collection.clone()),
            collection_nonce,
            &offer.quantity,
            &offer.payment_token,
            offer.payment_nonce,
            &nft_info.creator,
            &seller,
            &offer.owner,
            &self.calculate_global_offer_split(&offer, &nft_info),
            false
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
}
