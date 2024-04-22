multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use crate::auction::AuctionType;
#[multiversx_sc::module]
pub trait AdminModule:
    crate::storage::StorageModule
    + crate::helpers::HelpersModule
    + crate::views::ViewsModule
    + crate::events::EventsModule
    + crate::common::CommonModule
    + crate::wrapping::WrappingModule
{
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
        let map = self.reward_ticker();
        // require!(map.is_empty(), "The ticker was already set!");
        map.set(token);
    }

    #[only_owner]
    #[endpoint(setSpecialRewardAmount)]
    fn set_special_reward_amount(&self, token: &TokenIdentifier, amount: BigUint) {
        require!(
            &self.reward_ticker().get() == token,
            "The reward ticker is not used!"
        );
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

    #[endpoint(addWhitelist)]
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

    #[endpoint(removeWhitelist)]
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

    #[endpoint(unFreezeAllAuctionIds)]
    fn un_freeze_all_auction_id(&self) {
        self.require_admin(None);
        self.freezed_auctions().clear();
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
}
