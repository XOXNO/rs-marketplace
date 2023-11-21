multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use crate::auction::{AuctionType, CollectionExtraFeesConfig, CollectionFeeConfig};
#[multiversx_sc::module]
pub trait AdminModule:
    crate::storage::StorageModule
    + crate::helpers::HelpersModule
    + crate::views::ViewsModule
    + crate::events::EventsModule
    + crate::common::CommonModule
    + crate::wrapping::WrappingModule
{
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

    #[endpoint(returnListing)]
    fn return_listing(&self, auction_ids: MultiValueEncoded<u64>) {
        self.require_admin(None);
        for auction_id in auction_ids {
            let map_auction = self.auction_by_id(auction_id);
            if map_auction.is_empty() {
                continue;
            }
            let mut auction = map_auction.get();
            if auction.auction_type == AuctionType::SftOnePerPayment
                || auction.auction_type == AuctionType::Nft
            {
                self.withdraw_auction_common(auction_id, &auction);
            } else if auction.current_winner.is_zero() {
                self.end_auction_common(auction_id, &auction);
            } else {
                if auction.current_winner != ManagedAddress::zero() {
                    self.transfer_or_save_payment(
                        &auction.current_winner,
                        &auction.payment_token_type,
                        auction.payment_token_nonce,
                        &auction.current_bid,
                    );
                    self.listings_bids(&auction.current_winner)
                        .remove(&auction_id);

                    auction.current_winner = ManagedAddress::zero();
                    self.end_auction_common(auction_id, &auction);
                }
            }
        }
    }

    #[endpoint(withdrawGlobalOffers)]
    fn withdraw_global_offers(&self, offer_ids: MultiValueEncoded<u64>) {
        self.require_admin(None);
        for offer_id in offer_ids {
            let map_offer = self.global_offer(offer_id);
            if map_offer.is_empty() {
                continue;
            }
            let offer = map_offer.get();
            self.common_global_offer_remove(&offer, true);
            self.emit_remove_global_offer_event(offer_id, &offer.collection);
        }
    }

    #[endpoint(deleteOffersByWallet)]
    fn delete_user_offers(&self, user: &ManagedAddress) {
        self.require_admin(None);
        let offers_root = self.offers_by_wallet(user);
        if offers_root.len() > 0 {
            for offer in offers_root.iter().take(80) {
                self.common_withdraw_offer(offer, &self.offer_by_id(offer).get());
            }
        }
    }

    #[endpoint(cleanExpiredOffers)]
    fn clean_expired_offers(&self, offer_ids: MultiValueEncoded<u64>) {
        self.require_admin(None);
        let timestamp = self.blockchain().get_block_timestamp();
        for offer_id in offer_ids {
            let offer = self.offer_by_id(offer_id);
            if !offer.is_empty() {
                let main_offer = offer.get();
                if main_offer.deadline <= timestamp {
                    self.common_withdraw_offer(offer_id, &main_offer);
                }
            }
        }
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(addRewardBalance)]
    fn add_reward_balance(
        &self,
        #[payment_token] token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] amount: BigUint,
    ) {
        require!(
            self.reward_ticker().get() == token,
            "This token is not used for rewards!"
        );
        self.reward_balance().update(|qt| *qt += &amount);
    }

    #[only_owner]
    #[endpoint(setRewardTicker)]
    fn set_reward_ticker(&self, token: EgldOrEsdtTokenIdentifier) {
        require!(
            self.reward_ticker().is_empty(),
            "The ticker was already set!"
        );
        self.reward_ticker().set(token);
    }

    #[only_owner]
    #[endpoint(setSpecialRewardAmount)]
    fn set_special_reward_amount(&self, token: &TokenIdentifier, amount: BigUint) {
        self.special_reward_amount(token).set(amount);
    }

    #[only_owner]
    #[endpoint(setDefaultRewardAmount)]
    fn set_default_reward_amount(&self, amount: BigUint) {
        self.reward_amount().set(amount);
    }

    #[endpoint(setAcceptedTokens)]
    fn set_accepted_tokens(&self, token: EgldOrEsdtTokenIdentifier) {
        self.require_admin(None);
        self.accepted_tokens().insert(token);
    }

    #[endpoint(removeAcceptedTokens)]
    fn remove_accepted_tokens(&self, token: EgldOrEsdtTokenIdentifier) -> bool {
        self.require_admin(None);
        self.accepted_tokens().remove(&token)
    }

    #[endpoint(addWitelistedSC)]
    fn add_whitelisted_sc(&self, sc: ManagedAddress) {
        self.require_admin(None);
        require!(
            self.blockchain().is_smart_contract(&sc),
            "The address is not a smart contract!"
        );
        let mut tokens = self.claimable_tokens(&sc);
        for token in tokens.iter() {
            let mut nonces = self.claimable_token_nonces(&sc, &token);
            for nonce in nonces.iter() {
                let amount_map = self.claimable_amount(&sc, &token, nonce);
                let amount = amount_map.get();
                if amount > BigUint::zero() {
                    self.send().direct(&sc, &token, nonce, &amount_map.get());
                    amount_map.clear();
                }
            }
            nonces.clear();
        }
        tokens.clear();
        self.whitelisted_contracts().insert(sc);
    }

    #[endpoint(removeWitelistedSC)]
    fn remove_wl_sc(&self, sc: ManagedAddress) {
        self.require_admin(None);
        require!(
            self.blockchain().is_smart_contract(&sc),
            "The address is not a smart contract!"
        );
        self.whitelisted_contracts().remove(&sc);
    }

    #[endpoint(setStatus)]
    fn set_status(&self, status: bool) {
        self.require_admin(None);
        self.status().set(&status);
    }

    #[only_owner]
    #[endpoint(setCutPercentage)]
    fn set_percentage_cut(&self, new_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(new_cut_percentage)
    }

    #[endpoint(unFreezeAuctionId)]
    fn un_freeze_auction_id(&self, auction_id: u64) {
        self.require_admin(None);
        self.freezed_auctions().swap_remove(&auction_id);
    }

    #[endpoint(freezeAuctionId)]
    fn freeze_auction_id(&self, auction_id: u64) {
        self.require_admin(None);
        self.freezed_auctions().insert(auction_id);
    }

    #[only_owner]
    #[endpoint(claimLeftOverDust)]
    fn claim_lost_funds(&self, token: &EgldOrEsdtTokenIdentifier, amount: &BigUint) {
        self.send()
            .direct(&self.blockchain().get_owner_address(), token, 0, amount);
    }

    #[endpoint(claimSavedFundsForUser)]
    fn claim_tokens_for_creator(&self, wallet: &ManagedAddress) {
        self.require_admin(None);
        let mut tokens = self.claimable_tokens(wallet);
        for token in tokens.iter() {
            let mut nonces = self.claimable_token_nonces(wallet, &token);
            for nonce in nonces.iter() {
                let amount_map = self.claimable_amount(wallet, &token, nonce);
                let amount = amount_map.get();
                if amount > BigUint::zero() {
                    self.send().direct(wallet, &token, nonce, &amount_map.get());
                    amount_map.clear();
                }
            }
            nonces.clear();
        }
        tokens.clear();
    }

    #[endpoint(addBlackListWallet)]
    fn add_blacklist(&self, wallet: ManagedAddress) -> bool {
        self.require_admin(None);
        self.blacklist_wallets().insert(wallet)
    }

    #[endpoint(removeBlackListWallet)]
    fn remove_blacklist(&self, wallet: ManagedAddress) -> bool {
        self.require_admin(None);
        self.blacklist_wallets().remove(&wallet)
    }

    #[endpoint(setCutFeesReverted)]
    fn set_cut_fees_reverted(&self, token_id: &TokenIdentifier, value: bool) {
        self.require_admin(None);
        let config_map = self.collection_config(&token_id);
        if config_map.is_empty() {
            config_map.set(CollectionFeeConfig {
                reverse_cut_fees: value,
                reverse_royalties: false,
                custom_royalties: false,
                min_royalties: BigUint::zero(),
                max_royalties: BigUint::zero(),
                extra_fees: CollectionExtraFeesConfig {
                    amount: BigUint::zero(),
                    address: ManagedAddress::zero(),
                },
                admin: ManagedAddress::zero(),
            });
        } else {
            config_map.update(|f| {
                f.reverse_cut_fees = value;
            })
        }
    }

    #[endpoint(setRoyaltiesReverted)]
    fn set_royalties_reverted(&self, token_id: &TokenIdentifier, value: bool) {
        self.require_admin(None);
        let config_map = self.collection_config(&token_id);
        if config_map.is_empty() {
            self.require_admin(None);
            config_map.set(CollectionFeeConfig {
                reverse_cut_fees: false,
                reverse_royalties: value,
                custom_royalties: false,
                min_royalties: BigUint::zero(),
                max_royalties: BigUint::zero(),
                extra_fees: CollectionExtraFeesConfig {
                    amount: BigUint::zero(),
                    address: ManagedAddress::zero(),
                },
                admin: ManagedAddress::zero(),
            });
        } else {
            self.require_admin(Some(config_map.get().admin));
            config_map.update(|f| {
                f.reverse_royalties = value;
            })
        }
    }

    #[endpoint(setExtraFees)]
    fn set_extra_fees(&self, token_id: &TokenIdentifier, amount: BigUint, address: ManagedAddress) {
        let config_map = self.collection_config(&token_id);
        if config_map.is_empty() {
            self.require_admin(None);
            config_map.set(CollectionFeeConfig {
                reverse_cut_fees: false,
                reverse_royalties: false,
                custom_royalties: false,
                min_royalties: BigUint::zero(),
                max_royalties: BigUint::zero(),
                extra_fees: CollectionExtraFeesConfig {
                    amount: amount,
                    address: address,
                },
                admin: ManagedAddress::zero(),
            });
        } else {
            self.require_admin(Some(config_map.get().admin));
            config_map.update(|f| {
                f.extra_fees.amount = amount;
                f.extra_fees.address = address;
            })
        }
    }

    #[endpoint(setCustomRoyalties)]
    fn set_custom_royalties(
        &self,
        token_id: &TokenIdentifier,
        min: BigUint,
        max: BigUint,
        enabled: bool,
    ) {
        let config_map = self.collection_config(&token_id);
        require!(
            min <= max,
            "Min royalties must be lower than max royalties!"
        );
        if config_map.is_empty() {
            self.require_admin(None);
            config_map.set(CollectionFeeConfig {
                reverse_cut_fees: false,
                reverse_royalties: false,
                custom_royalties: enabled,
                min_royalties: BigUint::zero(),
                max_royalties: BigUint::zero(),
                extra_fees: CollectionExtraFeesConfig {
                    amount: BigUint::zero(),
                    address: ManagedAddress::zero(),
                },
                admin: ManagedAddress::zero(),
            });
        } else {
            self.require_admin(Some(config_map.get().admin));
            config_map.update(|f| {
                f.min_royalties = min;
                f.max_royalties = max;
                f.custom_royalties = enabled;
            })
        }
    }

    #[endpoint(setConfigAdmin)]
    fn set_config_admin(&self, token_id: &TokenIdentifier, admin: ManagedAddress) {
        self.require_admin(None);
        let config_map = self.collection_config(&token_id);
        if config_map.is_empty() {
            config_map.set(CollectionFeeConfig {
                reverse_cut_fees: false,
                reverse_royalties: false,
                custom_royalties: false,
                min_royalties: BigUint::zero(),
                max_royalties: BigUint::zero(),
                extra_fees: CollectionExtraFeesConfig {
                    amount: BigUint::zero(),
                    address: ManagedAddress::zero(),
                },
                admin,
            });
        } else {
            config_map.update(|f| {
                f.admin = admin;
            })
        }
    }
}
