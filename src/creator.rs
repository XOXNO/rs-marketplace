elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait CreatorModule: crate::storage::StorageModule {
    #[endpoint(claimTokens)]
    fn claim_tokens(
        &self,
        token_id: EgldOrEsdtTokenIdentifier,
        token_nonce: u64,
        claim_destination: ManagedAddress,
    ) {
        let caller = self.blockchain().get_caller();
        let amount_mapper = self.claimable_amount(&caller, &token_id, token_nonce);
        let amount = amount_mapper.get();

        if amount > 0 {
            amount_mapper.clear();

            self.send()
                .direct(&claim_destination, &token_id, token_nonce, &amount);
        }
    }
}
