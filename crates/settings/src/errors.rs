use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml error: {0}")]
    TomlEdit(#[from] toml_edit::TomlError),

    #[error("not writable: {0:?}")]
    NotWritable(crate::source::SourceKind),

    #[error("immutable key: {0}")]
    ImmutableKey(String),

    #[error("key not found: {0}")]
    KeyNotFound(String),

    #[error("invalid target: {0:?}")]
    InvalidTarget(Option<crate::source::SourceKind>),

    #[error("other: {0}")]
    Other(String),
}
