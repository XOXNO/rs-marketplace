multiversx_sc::imports!();
multiversx_sc::derive_imports!();
// use pair::ProxyTrait as _;

#[multiversx_sc::module]
pub trait DexModule: crate::storage::StorageModule {
    // #[proxy]
    // fn dex_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;

    fn swap_wegld_for_xoxno(&self, destination: &ManagedAddress, payment: EsdtTokenPayment) {
        // self.dex_proxy(self.swap_pair_xoxno().get())
            // .swap_tokens_fixed_input(self.xoxno_token().get(), BigUint::from(1u64))
            // .with_esdt_transfer(payment)
            // .async_call()
            // .with_callback(self.callbacks().forward_xoxno(destination))
            // .call_and_exit();
    }

    #[callback]
    fn forward_xoxno(
        &self,
        destination: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<EsdtTokenPayment>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(payment) => self.send().direct_esdt(
                &destination,
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            ),
            ManagedAsyncCallResult::Err(_err) => {
                let payment = self.call_value().single_esdt();
                self.send().direct_esdt(
                    destination,
                    &payment.token_identifier,
                    payment.token_nonce,
                    &payment.amount,
                );
            }
        }
    }
}
