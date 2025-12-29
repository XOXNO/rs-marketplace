multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use multiversx_wegld_swap_sc::ProxyTrait as _;

#[multiversx_sc::module]
pub trait WrappingModule: crate::storage::StorageModule {
    #[proxy]
    fn wegld_proxy(&self, sc_address: ManagedAddress)
        -> multiversx_wegld_swap_sc::Proxy<Self::Api>;

    fn wrap_egld(&self, amount: BigUint) {
        self.wegld_proxy(self.wrapping().get())
            .wrap_egld()
            .with_egld_transfer(amount)
            .sync_call()
    }

    fn unwrap_egld(&self, amount: BigUint) {
        self.wegld_proxy(self.wrapping().get())
            .unwrap_egld()
            .with_esdt_transfer(EsdtTokenPayment::new(
                self.wrapping_token().get(),
                0,
                amount,
            ))
            .sync_call()
    }
}
