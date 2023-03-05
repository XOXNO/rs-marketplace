multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait PoolsModule: crate::storage::StorageModule + crate::events::EventsModule {
    #[payable("EGLD")]
    #[endpoint(deposit)]
    fn deposit(&self) {
        let payment = self.call_value().egld_or_single_esdt();
        if payment.amount > 0 {
            let map_acc_tokens = self.accepted_tokens();
            require!(
                map_acc_tokens.contains(&payment.token_identifier),
                "The deposited token is not whitelisted!"
            );
            let caller = self.blockchain().get_caller();
            let map_user = self.user_funds(&caller, &payment.token_identifier, payment.token_nonce);
            if map_user.is_empty() {
                map_user.set(payment);
            } else {
                map_user.update(|f| f.amount += payment.amount);
            }
            self.emit_deposit_balance(&caller, &map_user.get());
        }
    }

    #[endpoint(withdrawDeposit)]
    fn withdraw_deposit(&self, token: &EgldOrEsdtTokenIdentifier, nonce: u64, amount: &BigUint) {
        let caller = self.blockchain().get_caller();
        let map_user = self.user_funds(&caller, token, nonce);
        if !map_user.is_empty() {
            let balance = map_user.get();
            require!(
                &balance.amount >= amount,
                "Your balance is under the requested amount!",
            );
            let clear = &balance.amount == amount;
            if clear {
                map_user.clear();
                self.emit_deposit_balance(
                    &caller,
                    &EgldOrEsdtTokenPayment::new(token.clone(), nonce, BigUint::zero()),
                );
            } else {
                map_user.update(|f| f.amount -= amount);
                self.emit_deposit_balance(&caller, &map_user.get());
            }
            self.send().direct(&caller, token, nonce, amount);
        }
    }

    fn has_balance_and_deduct(
        &self,
        buyer: &ManagedAddress,
        token: &EgldOrEsdtTokenIdentifier,
        nonce: u64,
        amount: &BigUint,
    ) {
        let map_user = self.user_funds(&buyer, token, nonce);
        require!(!map_user.is_empty(), "This user has no balance deposited!");

        let balance = map_user.get();
        require!(
            &balance.amount >= amount,
            "Your balance is under the requested amount!",
        );
        let clear = &balance.amount == amount;
        if clear {
            map_user.clear();
            self.emit_deposit_balance(
                buyer,
                &EgldOrEsdtTokenPayment::new(token.clone(), nonce, BigUint::zero()),
            );
        } else {
            map_user.update(|f| f.amount -= amount);
            self.emit_deposit_balance(buyer, &map_user.get());
        }
    }

    fn has_balance(
        &self,
        buyer: &ManagedAddress,
        token: &EgldOrEsdtTokenIdentifier,
        nonce: u64,
        amount: &BigUint,
    ) {
        let map_user = self.user_funds(&buyer, &token, nonce);
        require!(!map_user.is_empty(), "This user has no balance deposited!");

        let balance = map_user.get();
        require!(
            &balance.amount >= amount,
            "Your balance is under the requested amount!",
        );
    }

    #[view(userDeposit)]
    #[storage_mapper("userBalance")]
    fn user_funds(
        &self,
        user: &ManagedAddress,
        token: &EgldOrEsdtTokenIdentifier,
        nonce: u64,
    ) -> SingleValueMapper<EgldOrEsdtTokenPayment>;
}
