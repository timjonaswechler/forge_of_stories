use super::binding_adapter::BindingSpec;
use std::{any::TypeId, collections::BTreeMap, sync::LazyLock};

pub trait KeyBindingValidator: Send + Sync {
    fn action_type_id(&self) -> TypeId;
    fn validate(&self, binding: &BindingSpec) -> Result<(), String>;
}

pub struct KeyBindingValidatorRegistration(pub fn() -> Box<dyn KeyBindingValidator>);

// inventory::collect!(KeyBindingValidatorRegistration);

pub(crate) static KEY_BINDING_VALIDATORS: LazyLock<BTreeMap<TypeId, Box<dyn KeyBindingValidator>>> =
    LazyLock::new(|| {
        let mut validators = BTreeMap::new();
        // for validator_registration in inventory::iter::<KeyBindingValidatorRegistration> {
        //     let validator = validator_registration.0();
        //     validators.insert(validator.action_type_id(), validator);
        // }
        validators
    });
