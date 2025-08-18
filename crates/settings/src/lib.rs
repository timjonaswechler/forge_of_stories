//! Settings crate
//!
//! Provides a layered settings store with TOML-backed file sources and an
//! in-memory defaults source. Preserves formatting via `toml_edit` for file
//! sources and supports atomic writes and optional cross-process advisory
//! locking (via the `fs2` crate).

pub mod errors;
pub mod file_toml_source;
pub mod in_memory;
pub mod key_path;
pub mod simple_path_resolver;
pub mod source;
pub mod store;

pub use file_toml_source::FileTomlSource;
pub use in_memory::InMemoryDefaults;
pub use key_path::KeyPath;
pub use simple_path_resolver::SimplePathResolver;
pub use source::{SettingSource, SourceKind};
pub use store::{SettingMeta, SettingStore};
