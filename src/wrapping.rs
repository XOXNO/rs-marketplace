elrond_wasm::imports!();
elrond_wasm::derive_imports!();
use elrond_sc_wegld_swap::ProxyTrait as _;

#[elrond_wasm::module]
pub trait WrappingModule: crate::storage::StorageModule {
    #[proxy]
    fn wegld_proxy(&self, sc_address: ManagedAddress) -> elrond_sc_wegld_swap::Proxy<Self::Api>;

    fn wrap_egld(&self, amount: BigUint) {
        self.wegld_proxy(self.wrapping().get())
            .wrap_egld()
            .with_egld_transfer(amount)
            .execute_on_dest_context()
    }

    fn unwrap_egld(&self, amount: BigUint) {
        self.wegld_proxy(self.wrapping().get())
            .unwrap_egld()
            .with_esdt_transfer(EsdtTokenPayment::new(
                self.wrapping_token().get(),
                0,
                amount,
            ))
            .execute_on_dest_context()
    }
}
