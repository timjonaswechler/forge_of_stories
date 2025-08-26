use anyhow::{Result, anyhow};
use serde::Serialize;
use toml_edit::{DocumentMut, Item, Value};

pub fn set_key_to_serialized_item<T: Serialize>(
    doc: &mut DocumentMut,
    key: &str,
    value: &T,
) -> Result<()> {
    let item: Item = toml_edit::ser::to_item(value)?;
    doc[key] = item;
    Ok(())
}

pub(crate) fn parse_toml_value_snippet(snippet: &str) -> Result<Value> {
    // Erwartet z. B. { scope = "global" } oder 42 oder "text"
    let doc_str = format!("v = {}", snippet);
    let doc: DocumentMut = doc_str.parse().context("Failed to parse TOML snippet")?;
    doc["v"]
        .clone()
        .into_value()
        .ok_or_else(|| anyhow!("Snippet did not parse into a TOML value"))
}
