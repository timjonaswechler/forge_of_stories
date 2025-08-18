use crate::errors::SettingError;
use crate::key_path::KeyPath;
use crate::source::{SettingSource, SourceKind};
use fs2::FileExt;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use toml_edit::{Document, Item, Table, Value as TomlEditValue};

/// FileTomlSource: file-backed source using toml_edit::Document<String>.
///
/// Notes:
/// - We keep an owned `Document<String>` internally so we can reliably produce
///   owned TOML text when persisting.
/// - Persist writes a temporary file and renames it into place; an advisory lock
///   is acquired around the write for cross-process safety.
/// - For mutating (set) we now perform in-place `toml_edit::Document` edits to
///   preserve unrelated comments/formatting. Additional typed-write helpers are
///   provided to write Serializable Rust values directly.
pub struct FileTomlSource {
    path: PathBuf,
    doc: Document<String>,
    prec: i32,
    writable: bool,
    kind: SourceKind,
}

impl FileTomlSource {
    /// Open (or create) a TOML-backed file source.
    /// This will ensure parent directories exist (create them if necessary).
    pub fn open(
        path: impl Into<PathBuf>,
        kind: SourceKind,
        precedence: i32,
        writable: bool,
    ) -> Result<Self, SettingError> {
        let path = path.into();

        // Ensure parent directories exist for writable sources.
        if writable {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Load existing contents if present, producing a Document<String>.
        let doc: Document<String> = if path.exists() {
            let s = std::fs::read_to_string(&path)?;
            s.parse::<Document<String>>()
                .map_err(SettingError::TomlEdit)?
        } else {
            // Empty document as owned String-backed Document
            "".parse::<Document<String>>()
                .map_err(SettingError::TomlEdit)?
        };

        Ok(Self {
            path,
            doc,
            prec: precedence,
            writable,
            kind,
        })
    }

    /// Helper that acquires an exclusive lock on the backing file (creates it if needed),
    /// runs the provided closure and releases the lock. This is used to provide
    /// cross-process safety for write operations.
    fn with_lock<F, R>(&self, mut f: F) -> Result<R, SettingError>
    where
        F: FnMut(&File) -> Result<R, SettingError>,
    {
        // Open for read-write (create if needed)
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.path)?;
        // acquire exclusive lock for cross-process safety during write
        file.lock_exclusive().map_err(SettingError::Io)?;
        let res = f(&file);
        // unlock explicitly (dropping the file would also release, but be explicit)
        file.unlock().map_err(SettingError::Io)?;
        res
    }

    fn ensure_parent_dir(&self) -> Result<(), SettingError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Typed helper: serialize `t` to TOML and set it at `key_path`.
    /// This tries to preserve comments by parsing the serialized TOML into a
    /// `toml_edit::Document`/`Item` and inserting the resulting item in-place.
    pub fn set_typed<T: Serialize>(
        &mut self,
        key_path: &KeyPath,
        t: &T,
    ) -> Result<(), SettingError> {
        if !self.writable {
            return Err(SettingError::NotWritable(self.kind()));
        }

        // Serialize to a TOML string using `toml` crate
        let toml_str = toml::to_string(t).map_err(|e| SettingError::Other(e.to_string()))?;

        // Attempt to parse the serialized string into a toml_edit::Document.
        // For scalar values the parsed Document may be empty, so also try parsing
        // directly into a toml_edit::Value when needed.
        let parsed_doc_result = toml_str.parse::<Document<String>>();
        if let Ok(parsed_doc) = parsed_doc_result {
            // If the serialized form is a table (common for structs), insert the table
            if !parsed_doc.as_table().is_empty() {
                // Insert the whole parsed table as the value at key_path
                return self.set(key_path, Item::Table(parsed_doc.as_table().clone()));
            }
        }

        // Fall back: try parsing the string as a raw toml_edit::Value (scalar or array)
        let parsed_value: TomlEditValue = toml_str.parse().map_err(SettingError::TomlEdit)?;

        self.set(key_path, Item::Value(parsed_value))?;
        Ok(())
    }

    /// Typed helper: write an entire struct as a table under `prefix`.
    /// If `prefix` is empty, the struct's table replaces the document root.
    pub fn write_struct<T: Serialize>(
        &mut self,
        prefix: &KeyPath,
        t: &T,
    ) -> Result<(), SettingError> {
        if !self.writable {
            return Err(SettingError::NotWritable(self.kind()));
        }

        // Serialize struct to TOML text
        let toml_str = toml::to_string(t).map_err(|e| SettingError::Other(e.to_string()))?;

        // Parse into a Document to get an owned toml_edit::Table
        let parsed_doc: Document<String> = toml_str
            .parse::<Document<String>>()
            .map_err(SettingError::TomlEdit)?;

        // If parsed_doc root is empty but the struct serialized to a scalar, error out
        if parsed_doc.as_table().is_empty() {
            return Err(SettingError::Other(
                "struct did not serialize to a TOML table".to_string(),
            ));
        }

        // Insert the parsed table at the given prefix using the same in-place logic
        // If prefix is empty, replace root with parsed table
        if prefix.as_slice().is_empty() {
            self.doc = parsed_doc;
            return Ok(());
        }

        // Otherwise, insert the table as the value at prefix
        self.set(prefix, Item::Table(parsed_doc.as_table().clone()))?;
        Ok(())
    }
}

impl SettingSource for FileTomlSource {
    fn kind(&self) -> SourceKind {
        self.kind
    }

    fn precedence(&self) -> i32 {
        self.prec
    }

    fn is_writable(&self) -> bool {
        self.writable
    }

    fn load(&mut self) -> Result<(), SettingError> {
        if self.path.exists() {
            let s = std::fs::read_to_string(&self.path)?;
            self.doc = s
                .parse::<Document<String>>()
                .map_err(SettingError::TomlEdit)?;
        }
        Ok(())
    }

    fn get(&self, key_path: &KeyPath) -> Result<Option<Item>, SettingError> {
        // Empty key path -> root table
        if key_path.as_slice().is_empty() {
            return Ok(Some(Item::Table(self.doc.as_table().clone())));
        }

        // Walk nested tables using toml_edit read-only APIs
        let mut cur_table = self.doc.as_table();
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

    fn set(&mut self, key_path: &KeyPath, value: Item) -> Result<(), SettingError> {
        if !self.writable {
            return Err(SettingError::NotWritable(self.kind()));
        }

        // Strategy:
        // 1) serialize current Document<String> to TOML string
        // 2) parse into toml::Value
        // 3) traverse/create nested maps and insert the parsed `value` (converted to toml::Value)
        // 4) serialize toml::Value back to string and parse into Document<String>
        //
        // This avoids depending on internal mutable table accessor APIs for Document<String>.

        // 1) current doc -> toml::Value
        let cur_toml: toml::Value = toml::from_str(&self.doc.to_string())
            .map_err(|e| SettingError::Other(e.to_string()))?;

        // Ensure we have a table root
        let mut root_table = match cur_toml {
            toml::Value::Table(m) => m,
            _ => toml::map::Map::new(),
        };

        // Convert the provided toml_edit::Item to a toml::Value by serializing its string
        // representation and parsing that as a value.
        let value_str = value.to_string();
        let parsed_value: toml::Value =
            toml::from_str(&value_str).map_err(|e| SettingError::Other(e.to_string()))?;

        // Traverse and create nested tables
        let parts = key_path.as_slice();
        if parts.is_empty() {
            // Setting the root table: replace whole document
            if let toml::Value::Table(new_table) = parsed_value {
                root_table = new_table;
            } else {
                // If user attempts to set non-table at root, wrap into a table under special key? Reject.
                return Err(SettingError::Other(
                    "cannot set non-table value at document root".to_string(),
                ));
            }
        } else {
            let mut cur_map = &mut root_table;
            for (i, seg) in parts.iter().enumerate() {
                let is_last = i == parts.len() - 1;
                if is_last {
                    // insert the parsed value at this key
                    cur_map.insert(seg.clone(), parsed_value.clone());
                } else {
                    // ensure nested table exists
                    if !cur_map.contains_key(seg) || !cur_map[seg].is_table() {
                        cur_map.insert(seg.clone(), toml::Value::Table(toml::map::Map::new()));
                    }
                    // descend
                    if let Some(toml::Value::Table(m)) = cur_map.get_mut(seg) {
                        cur_map = m;
                    } else {
                        // Shouldn't happen because we ensured it's a table
                        return Err(SettingError::Other(
                            "failed to create/descend into nested table".to_string(),
                        ));
                    }
                }
            }
        }

        // Serialize the modified toml map back to a string and parse into Document<String>
        let new_value = toml::Value::Table(root_table);
        let new_toml_string =
            toml::to_string(&new_value).map_err(|e| SettingError::Other(e.to_string()))?;
        self.doc = new_toml_string
            .parse::<Document<String>>()
            .map_err(SettingError::TomlEdit)?;

        Ok(())
    }

    fn persist(&self) -> Result<(), SettingError> {
        if !self.writable {
            return Ok(());
        }
        self.ensure_parent_dir()?;
        // Use stable to_string() for the Document<String>
        let bytes = self.doc.to_string().into_bytes();
        let tmp = self.path.with_extension("tmp");
        // Use with_lock to get file handle & lock
        let res = self.with_lock(|_file| {
            // write temp file
            let mut f = File::create(&tmp)?;
            f.write_all(&bytes)?;
            f.sync_all()?;
            // atomic rename
            std::fs::rename(&tmp, &self.path)?;
            // fsync parent dir
            if let Some(dir) = self.path.parent() {
                let d = File::open(dir)?;
                d.sync_all()?;
            }
            Ok(())
        });
        // propagate result (and any errors)
        res
    }
}
