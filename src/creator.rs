use crate::{CollectionExtraFeesConfig, CollectionFeeConfig};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait CreatorModule:
    crate::storage::StorageModule
    + crate::helpers::HelpersModule
    + crate::views::ViewsModule
    + crate::events::EventsModule
    + crate::wrapping::WrappingModule
{
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
        self.emit_collection_config(token_id, &config_map.get());
    }

    #[endpoint(setRoyaltiesReverted)]
    fn set_royalties_reverted(&self, token_id: &TokenIdentifier, value: bool) {
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
        self.emit_collection_config(token_id, &config_map.get());
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
        self.emit_collection_config(token_id, &config_map.get());
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
            "Min royalties must be lower or equal than max royalties!"
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
        self.emit_collection_config(token_id, &config_map.get());
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
        self.emit_collection_config(token_id, &config_map.get());
    }
}
