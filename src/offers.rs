multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use core::convert::TryInto;

use super::auction::{AuctionType, Offer, OfferStatus};
use crate::auction::GlobalOffer;
use crate::common;
use crate::events;
use crate::helpers;
use crate::pools;
use crate::views;
use crate::wrapping;
use crate::{storage, NFT_AMOUNT};

#[multiversx_sc::module]
pub trait CustomOffersModule:
    storage::StorageModule
    + helpers::HelpersModule
    + events::EventsModule
    + views::ViewsModule
    + common::CommonModule
    + wrapping::WrappingModule
    + pools::PoolsModule
{
    #[payable("*")]
    #[endpoint(acceptOffer)]
    fn accept_offer(&self, offer_id: u64, auction_id: OptionalValue<u64>) {
        self.require_enabled();
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
        if offer.new_version {
            self.has_balance_and_deduct(
                &offer.offer_owner,
                &offer.payment_token_type,
                offer.payment_token_nonce,
                &offer.price,
            );
        }

        let auction_id_sent = auction_id.clone().into_option().unwrap_or(0);

        if auction_id.is_none() {
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
        } else {
            let auction = self.try_get_auction(auction_id_sent);
            self.common_offer_auction_check(&offer, &auction);

            require!(
                seller == auction.original_owner,
                "Just the owner of the listed NFT can accept the offer!"
            );

            self.update_or_remove_items_quantity(&auction, &auction.nr_auctioned_tokens);
            self.remove_auction_common(auction_id_sent, &auction);
        }

        offer.status = OfferStatus::Accepted;
        let nft_info = self.get_nft_info(&offer.token_type, offer.token_nonce);
        let creator_royalties_percentage = nft_info.royalties;

        self.common_offer_remove(offer_id, &offer);
        self.emit_accept_offer_event(offer_id, &offer, &seller, auction_id_sent);
        self.distribute_tokens_common(
            ManagedVec::from(EsdtTokenPayment::new(
                offer.token_type.clone(),
                offer.token_nonce,
                offer.quantity.clone(),
            )),
            &offer.payment_token_type,
            offer.payment_token_nonce,
            &nft_info.creator,
            &seller,
            &offer.offer_owner,
            &self.calculate_amount_split(
                &offer.price,
                &creator_royalties_percentage,
                self.get_collection_config(&offer.token_type),
            ),
            false,
        );
    }

    #[payable("*")]
    #[endpoint(declineOffer)]
    fn decline_offer(&self, offer_id: u64, auction_id: OptionalValue<u64>) {
        self.require_enabled();
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
                payment_token.is_egld() && payment_amount.eq(&BigUint::zero()),
                "You have to send 0 eGLD as payment to decline the offer!"
            );

            let auction = self.try_get_auction(auction_id.into_option().unwrap());
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
        self.require_enabled();
        let offer = self.try_get_offer(offer_id);
        let caller = self.blockchain().get_caller();

        require!(
            offer.offer_owner == caller,
            "Only the original owner can withdraw the offer!"
        );

        self.common_withdraw_offer(offer_id, &offer);
    }

    #[payable("EGLD")]
    #[endpoint(sendOffer)]
    fn send_offer(
        &self,
        payment_token: EgldOrEsdtTokenIdentifier,
        payment_token_nonce: u64,
        payment_amount: BigUint,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        nft_amount: BigUint,
        deadline: u64,
    ) -> u64 {
        self.require_enabled();

        require!(
            self.accepted_tokens().contains(&payment_token),
            "The payment token is not whitelisted!"
        );

        require!(payment_token.is_egld(), "The payment token is not EGLD!");
        require!(payment_token_nonce == 0, "The payment nonce is not 0!");
        require!(
            nft_nonce > 0,
            "Only Semi-Fungible and Non-Fungible tokens can have offers"
        );
        require!(
            nft_amount == BigUint::from(NFT_AMOUNT),
            "The quantity has to be 1!"
        );
        self.deposit();
        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();
        self.has_balance(
            &caller,
            &payment_token,
            payment_token_nonce,
            &payment_amount,
        );

        require!(
            !self.blacklist_wallets().contains(&caller),
            "Your address was blacklisted, all your SCAM offers are lost!"
        );
        let map_offer_check = self.check_offer_sent(&caller, &nft_type, nft_nonce, &payment_token);
        require!(
            !map_offer_check.get(),
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
            new_version: true,
        };
        // Map ID with Offer Struct
        self.offer_by_id(offer_id).set(&offer);
        self.token_offers_ids(&nft_type, nft_nonce).insert(offer_id);
        // Push ID to the offers list
        self.offers().insert(offer_id);
        // Add to the owner wallet the new Offer ID
        self.offers_by_wallet(&offer.offer_owner).insert(offer_id);
        map_offer_check.set(&true);
        // Emit event for new offer
        self.emit_offer_token_event(offer_id, offer);

        offer_id
    }

    #[payable("EGLD")]
    #[endpoint(sendGlobalOffer)]
    fn send_global_offer(
        &self,
        payment_token: EgldOrEsdtTokenIdentifier,
        payment_nonce: u64,
        price: BigUint,
        collection: TokenIdentifier,
        quantity: BigUint,
        attributes: OptionalValue<ManagedBuffer>,
    ) -> u64 {
        self.require_enabled();
        self.deposit();
        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();
        self.has_balance(&caller, &payment_token, payment_nonce, &price);
        let mut map_count_user_offers = self.user_global_offers(&caller);
        require!(
            map_count_user_offers.len() <= 250,
            "You can not place over 250 global offers per wallet!"
        );
        require!(
            !self.blacklist_wallets().contains(&caller),
            "Your address was blacklisted!"
        );
        let mut user_map = self.user_collection_global_offers(&caller, &collection);
        require!(
            user_map.len() <= 25,
            "You have a limit of 25 offers per collection!"
        );

        let offer_id = self.last_valid_global_offer_id().get() + 1;
        let offer = GlobalOffer {
            offer_id,
            collection: collection.clone(),
            quantity: quantity,
            payment_token,
            payment_nonce,
            price,
            timestamp: current_time,
            owner: caller.clone(),
            attributes: attributes.into_option(),
            new_version: true,
        };
        self.last_valid_global_offer_id().set(&offer_id);

        self.collection_global_offers(&collection).insert(offer_id);
        map_count_user_offers.insert(offer_id);
        user_map.insert(offer_id);
        self.emit_send_global_offer_event(&offer);
        self.global_offer_ids().insert(offer_id);
        self.global_offer(offer_id).set(offer);
        offer_id
    }

    #[endpoint(withdrawGlobalOffer)]
    fn withdraw_global_offer(&self, offer_id: u64) {
        self.require_enabled();
        let caller = self.blockchain().get_caller();
        let offer = self.try_get_global_offer(offer_id);
        require!(
            offer.owner.eq(&caller),
            "You are not the owner of this offer!"
        );
        self.common_global_offer_remove(&offer, true);
    }

    #[allow_multiple_var_args]
    #[payable("*")]
    #[endpoint(acceptGlobalOffer)]
    fn accept_global_offer(
        &self,
        offer_id: u64,
        auction_id_opt: OptionalValue<ManagedVec<u64>>,
        signature: OptionalValue<ManagedBuffer>,
    ) {
        self.require_enabled();
        let nfts = self.call_value().all_esdt_transfers().clone_value();
        let offer_map = self.global_offer(offer_id);
        let auctions_ids = auction_id_opt.into_option().unwrap_or(ManagedVec::new());
        let mut total_quantity_wanted = BigUint::zero();
        require!(!offer_map.is_empty(), "This offer is already removed!");
        let seller = self.blockchain().get_caller();
        let mut offer = offer_map.get();

        let mut tmp_nonces = ManagedBuffer::new();
        let mut accepted_nfts: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        let mut last_nft_info: EsdtTokenData = EsdtTokenData::default();
        for nft in nfts.iter() {
            require!(
                offer.collection.eq(&nft.token_identifier),
                "The collection sent is not the offer requested one!"
            );
            total_quantity_wanted += &nft.amount;
            if last_nft_info.creator == ManagedAddress::zero() {
                last_nft_info = self.get_nft_info(&offer.collection, nft.token_nonce);
            }
            if offer.attributes.is_some() {
                tmp_nonces.append(&self.decimal_to_ascii(nft.token_nonce.try_into().unwrap()));
            }
            accepted_nfts.push(nft);
        }

        for auction_id in auctions_ids.iter() {
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
                auction.nr_auctioned_tokens == BigUint::from(1u32),
                "The quantity listed is not matching the offer!"
            );

            require!(
                auction.auctioned_token_type == offer.collection,
                "The listed token is not matching the offer!"
            );

            if last_nft_info.creator == ManagedAddress::zero() {
                last_nft_info = self.get_nft_info(&offer.collection, auction.auctioned_token_nonce);
            }

            if offer.attributes.is_some() {
                tmp_nonces.append(
                    &self.decimal_to_ascii(auction.auctioned_token_nonce.try_into().unwrap()),
                );
            }

            self.update_or_remove_items_quantity(&auction, &auction.nr_auctioned_tokens);
            self.remove_auction_common(auction_id, &auction);

            total_quantity_wanted += &auction.nr_auctioned_tokens;

            accepted_nfts.push(EsdtTokenPayment::new(
                auction.auctioned_token_type,
                auction.auctioned_token_nonce,
                auction.nr_auctioned_tokens,
            ));
        }

        require!(
            &offer.quantity >= &total_quantity_wanted,
            "The offer is not accepting more than {} items",
            (offer.quantity)
        );

        let to_deduct_payment_amount = &offer.price.clone().mul(&total_quantity_wanted);

        if offer.new_version {
            self.has_balance_and_deduct(
                &offer.owner,
                &offer.payment_token,
                offer.payment_nonce,
                to_deduct_payment_amount,
            );
        }

        if offer.attributes.is_some() {
            let sign = signature.into_option();
            require!(sign.is_some(), "Signature required!");
            let mut data = ManagedBuffer::new();
            data.append(seller.as_managed_buffer());
            data.append(offer.collection.as_managed_buffer());
            data.append(&tmp_nonces);
            data.append(&self.decimal_to_ascii(offer.offer_id.try_into().unwrap()));
            data.append(&offer.attributes.as_ref().unwrap());

            let signer: ManagedAddress = self.signer().get();
            self.crypto()
                .verify_ed25519(signer.as_managed_buffer(), &data, &sign.unwrap());
        }
        self.common_global_offer_remove(&offer, false);

        self.emit_accept_global_offer_event(
            &offer,
            &seller,
            &accepted_nfts,
            &total_quantity_wanted,
            &auctions_ids,
        );

        self.distribute_tokens_common(
            accepted_nfts,
            &offer.payment_token,
            offer.payment_nonce,
            &last_nft_info.creator,
            &seller,
            &offer.owner,
            &self.calculate_amount_split(
                to_deduct_payment_amount,
                &last_nft_info.royalties,
                self.get_collection_config(&offer.collection),
            ),
            false,
        );

        if &offer.quantity != &total_quantity_wanted {
            offer.quantity -= &total_quantity_wanted;
            offer_map.set(offer.clone());
        }
    }
}
