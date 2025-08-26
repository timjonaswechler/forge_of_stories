use super::binding_adapter::BindingSpec;
#[derive(Debug)]
#[must_use]
pub enum KeymapFileLoadResult {
    Success {
        key_bindings: Vec<BindingSpec>,
    },
    SomeFailedToLoad {
        key_bindings: Vec<BindingSpec>,
        error_message: String,
    },
    TomlParseFailure {
        error: anyhow::Error,
    },
}
