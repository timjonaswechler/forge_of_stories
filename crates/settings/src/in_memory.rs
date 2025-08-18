use crate::errors::SettingError;
use crate::key_path::KeyPath;
use crate::source::{SettingSource, SourceKind};
use toml_edit::{Document, Item, Table};

/// InMemoryDefaults: read-only source that stores an owned `Document<String>`.
///
/// We intentionally store a `Document<String>` (owned backing) so merging with
/// file-backed `Document<String>` instances is consistent and so we can reliably
/// serialize to TOML text later if needed.
///
/// Constructing from other `Document<S>` variants is supported via `from_document`.
pub struct InMemoryDefaults {
    pub doc: Document<String>,
    pub prec: i32,
}

impl InMemoryDefaults {
    /// Construct from any `Document<S>` by converting it into an owned
    /// `Document<String>`. If the conversion/parsing fails we fall back to an
    /// empty document.
    pub fn from_document<S>(doc: Document<S>, precedence: i32) -> Self {
        // Convert via string round-trip so we get an owned-String Document.
        let owned = doc
            .to_string()
            .parse::<Document<String>>()
            .unwrap_or_else(|_| {
                "".parse::<Document<String>>()
                    .expect("parsing empty doc failed")
            });
        Self {
            doc: owned,
            prec: precedence,
        }
    }

    /// Convenience constructor for an empty defaults document.
    pub fn empty(precedence: i32) -> Self {
        let doc = ""
            .parse::<Document<String>>()
            .expect("parsing empty doc failed");
        Self {
            doc,
            prec: precedence,
        }
    }
}

impl SettingSource for InMemoryDefaults {
    fn kind(&self) -> SourceKind {
        SourceKind::Defaults
    }

    fn precedence(&self) -> i32 {
        self.prec
    }

    fn is_writable(&self) -> bool {
        false
    }

    fn load(&mut self) -> Result<(), SettingError> {
        // In-memory defaults are already loaded.
        Ok(())
    }

    fn get(&self, key_path: &KeyPath) -> Result<Option<Item>, SettingError> {
        // Empty key path -> return root table
        if key_path.is_empty() {
            return Ok(Some(Item::Table(self.doc.as_table().clone())));
        }

        // Walk nested tables for the given key path
        let mut cur_table: &Table = self.doc.as_table();
        let parts = key_path.as_slice();
        for (i, seg) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            if is_last {
                return Ok(cur_table.get(seg).cloned());
            }

            match cur_table.get(seg) {
                Some(next) if next.is_table() => {
                    if let Some(t) = next.as_table() {
                        cur_table = t;
                    } else {
                        return Ok(None);
                    }
                }
                _ => return Ok(None),
            }
        }

        Ok(None)
    }

    fn set(&mut self, _key_path: &KeyPath, _value: Item) -> Result<(), SettingError> {
        Err(SettingError::NotWritable(self.kind()))
    }

    fn persist(&self) -> Result<(), SettingError> {
        // In-memory defaults are not persisted.
        Ok(())
    }
}
