elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait AdminModule:
    crate::storage::StorageModule
    + crate::helpers::HelpersModule
    + crate::views::ViewsModule
    + crate::events::EventsModule
    + crate::common::CommonModule
    + crate::wrapping::WrappingModule
    + crate::dex::DexModule
{
    #[only_owner]
    #[endpoint(returnListing)]
    fn return_listing(&self, auction_id: u64) {
        let auction = self.try_get_auction(auction_id);
        self.withdraw_auction_common(auction_id, &auction);
        self.emit_withdraw_event(auction_id, &auction);
    }

    #[only_owner]
    #[endpoint(withdrawGlobalOffers)]
    fn withdraw_global_offers(&self, offer_id: u64) {
        require!(self.status().get(), "Global operation enabled!");
        let offer = self.try_get_global_offer(offer_id);
        self.common_global_offer_remove(&offer, true);
        self.emit_remove_global_offer_event(offer_id);
    }

    #[only_owner]
    #[endpoint(deleteOffersByWallet)]
    fn delete_user_offers(&self, user: &ManagedAddress) {
        let offers_root = self.offers_by_wallet(user);
        if offers_root.len() > 0 {
            for offer in offers_root.iter().take(80) {
                self.common_withdraw_offer(offer, &self.offer_by_id(offer).get());
            }
        }
    }

    #[only_owner]
    #[endpoint(cleanExpiredOffers)]
    fn clean_expired_offers(&self) -> i32 {
        let timestamp = self.blockchain().get_block_timestamp();
        let mut found = 0;
        for offer_id in self.offers().iter() {
            let offer = self.offer_by_id(offer_id);
            if !offer.is_empty() {
                let main_offer = offer.get();
                if main_offer.deadline < timestamp {
                    found += 1;
                    self.common_withdraw_offer(offer_id, &main_offer);
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

    #[only_owner]
    #[endpoint(setAcceptedTokens)]
    fn set_accepted_tokens(&self, token: EgldOrEsdtTokenIdentifier) {
        self.accepted_tokens().insert(token);
    }

    #[only_owner]
    #[endpoint(removeAcceptedTokens)]
    fn remove_accepted_tokens(&self, token: EgldOrEsdtTokenIdentifier) -> bool {
        self.accepted_tokens().remove(&token)
    }

    #[only_owner]
    #[endpoint(addWitelistedSC)]
    fn add_whitelisted_sc(&self, sc: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&sc),
            "The address is not a smart contract!"
        );
        self.whitelisted_contracts().insert(sc.clone());
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
    }

    #[only_owner]
    #[endpoint(removeWitelistedSC)]
    fn remove_wl_sc(&self, sc: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&sc),
            "The address is not a smart contract!"
        );
        self.whitelisted_contracts().remove(&sc);
    }

    #[only_owner]
    #[endpoint(setStatus)]
    fn set_status(&self, status: bool) {
        self.status().set(&status);
    }

    #[only_owner]
    #[endpoint(setCutPercentage)]
    fn set_percentage_cut(&self, new_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(new_cut_percentage)
    }

    #[only_owner]
    #[endpoint(claimTokensForCreator)]
    fn claim_tokens_for_creator(
        &self,
        token_id: EgldOrEsdtTokenIdentifier,
        token_nonce: u64,
        creator: ManagedAddress,
    ) {
        let amount_mapper = self.claimable_amount(&creator, &token_id, token_nonce);
        let amount = amount_mapper.get();

        if amount > 0 {
            amount_mapper.clear();
            let caller = self.blockchain().get_caller();
            self.send().direct(&caller, &token_id, token_nonce, &amount);
        }
    }

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
}
