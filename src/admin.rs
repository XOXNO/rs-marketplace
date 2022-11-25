use crate::auction::AuctionType;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait AdminModule:
    crate::storage::StorageModule
    + crate::helpers::HelpersModule
    + crate::views::ViewsModule
    + crate::events::EventsModule
{
    #[only_owner]
    #[endpoint(returnListing)]
    fn return_listing(&self, auction_id: u64) {
        let mut auction = self.try_get_auction(auction_id);

        require!(
            auction.current_winner.is_zero()
                || auction.auction_type == AuctionType::SftOnePerPayment
                || auction.auction_type == AuctionType::Nft,
            "Cannot withdraw, the auction already has bids!"
        );
        auction.current_winner = ManagedAddress::zero();
        self.distribute_tokens(&auction, Option::Some(&auction.nr_auctioned_tokens));

        self.token_auction_ids(
            auction.auctioned_token_type.clone(),
            auction.auctioned_token_nonce.clone(),
        )
        .remove(&auction_id);
        self.listings_by_wallet(auction.original_owner.clone())
            .remove(&auction_id);
        self.listings().remove(&auction_id);
        self.auction_by_id(auction_id).clear();
        self.emit_withdraw_event(auction_id, auction);
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(addRewardBalance)]
    fn add_reward_balance(
        &self,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] amount: BigUint,
    ) {
        require!(
            self.reward_ticker().get() == token,
            "This token is not used for rewards!"
        );
        self.reward_balance().update(|qt| *qt += &amount.clone());
    }

    #[only_owner]
    #[endpoint(setRewardTicker)]
    fn set_reward_ticker(&self, token: TokenIdentifier) {
        require!(
            self.reward_ticker().is_empty(),
            "The ticker was already set!"
        );
        self.reward_ticker().set(token);
    }

    #[only_owner]
    #[endpoint(setSpecialRewardAmount)]
    fn set_special_reward_amount(&self, token: TokenIdentifier, amount: BigUint) {
        self.special_reward_amount(token).set(amount);
    }

    #[only_owner]
    #[endpoint(setDefaultRewardAmount)]
    fn set_default_reward_amount(&self, amount: BigUint) {
        self.reward_amount().set(amount);
    }

    #[only_owner]
    #[endpoint(setAcceptedTokens)]
    fn set_accepted_tokens(&self, token: TokenIdentifier) {
        self.accepted_tokens().insert(token);
    }

    #[only_owner]
    #[endpoint(removeAcceptedTokens)]
    fn remove_accepted_tokens(&self, token: TokenIdentifier) -> bool {
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
                    self.send()
                        .direct(&sc, &token, nonce, &amount_map.get(), &[]);
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
}
