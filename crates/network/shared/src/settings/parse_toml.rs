//! TOML editing utilities using tree-sitter (feature-parity-oriented with the JSON helpers).
//!
//! Goals implemented here:
//! - update_value_in_toml_text: Minimal-invasive Diff/Update (similar Idee wie JSON-Version)
//! - replace_array_element_in_toml_text / append_array_element_in_toml_text for normal arrays
//! - replace_array_of_tables_entry_in_toml_text / append_array_of_tables_entry_in_toml_text
//!   for `[[array.of.tables]]` Blöcke (Index-bezogen).
//! - rename_key_in_toml_text: dediziertes Key-Renaming mit Erhalt von Kommentaren / Whitespace
//! - Kommentar / Whitespace Erhalt: Wert-Ersetzungen betreffen nur Value-Byte-Range;
//!   Inline-Kommentare und Surrounding-Spaces bleiben bestehen.
//! - to_pretty_toml + parse_toml_with_comments analog zu JSON-Pendants
//!
//! Hinweise / Grenzen:
//! - Dotted Keys werden als verschachtelte Tabellen interpretiert, beim Erstellen benutzen
//!   wir Tabellen-Header `[a.b]` statt am Root `a.b = ...` – kann angepasst werden.
//! - Bei neuem Erstellen fehlender Tabellen werden sie ans Dateiende gehängt.
//! - Arrays-of-Tables Anfügen: Block `[[path]]` + Key/Value Paare (hier nur ein Wert bei Append-API).
//! - Für weitergehende Format-Treue (Original-Spacing exakt spiegeln) ließen sich zusätzliche
//!   Heuristiken einbauen oder Snapshot-basiertes Re-Indenting.
//!
//! Alle öffentlichen Funktionen liefern entweder ein Edit (Range + Replacement) oder wenden
//! das Edit direkt an (Update-Funktion), analog zum existierenden JSON-Workflow.

use anyhow::{Context, Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::str;
use std::sync::LazyLock;
use tree_sitter::{Node, Parser, Query, QueryCursor};

/// A single text edit (byte-range, replacement text).
#[derive(Debug, Clone)]
pub struct TomlEdit {
    pub range: Range<usize>,
    pub replacement: String,
}

/// Query: Key/Value Paare.
static KEY_VALUE_QUERY: LazyLock<Query> = LazyLock::new(|| {
    Query::new(
        &tree_sitter_toml::language().into(),
        "(key_value_pair key: (_) @key value: (_) @value)",
    )
    .expect("KEY_VALUE_QUERY failed")
});

/// Query: Tabellen & Arrays-of-tables Header (dotted_key).
static TABLE_HEADER_QUERY: LazyLock<Query> = LazyLock::new(|| {
    Query::new(
        &tree_sitter_toml::language().into(),
        r#"
        (table (table_header (dotted_key) @table_path))
        (table_array_element (table_header (dotted_key) @table_path))
        "#,
    )
    .expect("TABLE_HEADER_QUERY failed")
});

/// Public API: Pretty printing (simple).
pub fn to_pretty_toml(
    value: &impl Serialize,
    _indent_size: usize,
    _indent_prefix: usize,
) -> String {
    toml::to_string(value).unwrap_or_else(|_| "## <serialization error>".into())
}

/// Public API: Parse TOML with comments ignored by standard TOML parsing.
pub fn parse_toml_with_comments<T: DeserializeOwned>(content: &str) -> Result<T> {
    Ok(toml::from_str(content)?)
}

/// Diff-orientierte Aktualisierung analog `update_value_in_json_text`.
///
/// - Traversiert rekursiv alte und neue toml::Value Tabellen.
/// - Erzeugt / löscht Keys.
/// - `preserved_keys`: Wenn Leaf-Key enthalten, wird Wert nicht angerührt.
/// - `rename_map`: Falls ein Key (aktueller Leaf) in dieser Map vorkommt, wird er
///   beim Ersetzen/Erstellen umbenannt (Rename Edit).
/// - Edits werden direkt im `text` angewendet und zusätzlich in `edits` gesammelt.
pub fn update_value_in_toml_text<'a>(
    text: &mut String,
    key_path: &mut Vec<&'a str>,
    tab_size: usize,
    old_value: &'a toml::Value,
    new_value: &'a toml::Value,
    preserved_keys: &[&str],
    rename_map: &HashMap<String, String>,
    edits: &mut Vec<TomlEdit>,
) {
    use toml::Value;

    match (old_value, new_value) {
        (Value::Table(old_tbl), Value::Table(new_tbl)) => {
            // Entfernte oder geänderte Keys
            for (k, old_sub) in old_tbl.iter() {
                key_path.push(k);
                if let Some(new_sub) = new_tbl.get(k) {
                    update_value_in_toml_text(
                        text,
                        key_path,
                        tab_size,
                        old_sub,
                        new_sub,
                        preserved_keys,
                        rename_map,
                        edits,
                    );
                } else {
                    // Entfernen
                    let (range, replacement) =
                        replace_value_in_toml_text(text, key_path, None, None, tab_size);
                    if range.start != range.end {
                        text.replace_range(range.clone(), &replacement);
                        edits.push(TomlEdit { range, replacement });
                    }
                }
                key_path.pop();
            }
            // Neue Keys
            for (k, new_sub) in new_tbl.iter() {
                if !old_tbl.contains_key(k) {
                    key_path.push(k);
                    update_value_in_toml_text(
                        text,
                        key_path,
                        tab_size,
                        &Value::String("__MISSING__".into()),
                        new_sub,
                        preserved_keys,
                        rename_map,
                        edits,
                    );
                    key_path.pop();
                }
            }
        }
        _ => {
            // Leaf
            if let Some(last) = key_path.last() {
                if preserved_keys.contains(last) {
                    return;
                }
            }
            if old_value != new_value {
                // Rename?
                let mut replace_key: Option<&str> = None;
                if let Some(last) = key_path.last() {
                    if let Some(new_key_name) = rename_map.get(*last) {
                        replace_key = Some(new_key_name.as_str());
                    }
                }
                let (range, replacement) = replace_value_in_toml_text(
                    text,
                    key_path,
                    Some(new_value),
                    replace_key,
                    tab_size,
                );
                if range.start != range.end || !replacement.is_empty() {
                    text.replace_range(range.clone(), &replacement);
                    edits.push(TomlEdit { range, replacement });
                }
            } else if let Some(last) = key_path.last() {
                // Falls nur Rename erwünscht (Inhalt gleich)
                if let Some(new_key_name) = rename_map.get(*last) {
                    if new_key_name.as_str() != *last {
                        let (range, replacement) = replace_value_in_toml_text(
                            text,
                            key_path,
                            Some(old_value),
                            Some(new_key_name),
                            tab_size,
                        );
                        if range.start != range.end {
                            text.replace_range(range.clone(), &replacement);
                            edits.push(TomlEdit { range, replacement });
                        }
                    }
                }
            }
        }
    }
}

/// Umbenennen eines Keys nur (ohne seinen Wert zu verändern).
///
/// table_path - Pfad zur Tabelle (leer => Root / vor erster Tabelle)
/// old_key -> new_key
pub fn rename_key_in_toml_text(
    text: &mut String,
    table_path: &[&str],
    old_key: &str,
    new_key: &str,
    tab_size: usize,
) -> Result<Option<TomlEdit>> {
    let mut path: Vec<&str> = table_path.iter().cloned().collect();
    path.push(old_key);
    let (range, replacement) =
        replace_value_in_toml_text(text, &path, None, Some(new_key), tab_size);
    if range.start == range.end && replacement.is_empty() {
        return Ok(None);
    }
    text.replace_range(range.clone(), &replacement);
    Ok(Some(TomlEdit { range, replacement }))
}

/// Ersetzt (oder löscht) ein Array-Element für Array-Wert den ein Key (key_path) adressiert.
/// new_value = None => löschen.
/// Ähnlich JSON `replace_top_level_array_value_in_json_text`.
pub fn replace_array_element_in_toml_text(
    text: &str,
    key_path: &[&str],
    array_index: usize,
    new_value: Option<&toml::Value>,
    tab_size: usize,
) -> Result<(Range<usize>, String)> {
    let (array_node, array_range) = find_array_value_node(text, key_path)
        .ok_or_else(|| anyhow!("Array for key path {:?} not found", key_path))?;

    // Sammle elementare Value-Knoten (überspringe Klammern / Kommata / comments)
    let mut elements: Vec<Node> = Vec::new();
    let mut ix = 0;
    while ix < array_node.child_count() {
        let ch = array_node.child(ix).unwrap();
        if !matches!(ch.kind(), "[" | "]" | "," | "comment") && !ch.is_missing() && !ch.is_error() {
            elements.push(ch);
        }
        ix += 1;
    }

    if array_index > elements.len() {
        // Append
        if new_value.is_none() {
            return Ok((0..0, String::new()));
        }
        return append_array_element_in_toml_text(text, key_path, new_value.unwrap(), tab_size);
    }
    if array_index == elements.len() {
        if new_value.is_none() {
            return Ok((0..0, String::new()));
        }
        return append_array_element_in_toml_text(text, key_path, new_value.unwrap(), tab_size);
    }

    let target = elements[array_index];
    let mut replace_range = target.byte_range();

    if new_value.is_none() {
        // Löschen + Komma entscheiden
        let arr_text = &text[array_range.clone()];
        let rel_start = replace_range.start - array_range.start;
        let rel_end = replace_range.end - array_range.start;

        // Versuch folgendes Komma einzusammeln
        if let Some(after) = arr_text.get(rel_end..) {
            if let Some(pos) = after.find(',') {
                let between = &after[..pos];
                if between.trim().is_empty() {
                    replace_range.end = array_range.start + rel_end + pos + 1;
                    return Ok((replace_range, String::new()));
                }
            }
        }
        // Sonst schau nach vorangehendem Komma
        if let Some(before) = arr_text.get(..rel_start) {
            if let Some(last_comma) = before.rfind(',') {
                if before[last_comma + 1..].trim().is_empty() {
                    replace_range.start = array_range.start + last_comma;
                }
            }
        }
        Ok((replace_range, String::new()))
    } else {
        let serialized = serialize_toml_value(new_value.unwrap(), tab_size);
        Ok((replace_range, serialized))
    }
}

/// Fügt ein Element an ein existierendes Array (oder erstellt das Array).
pub fn append_array_element_in_toml_text(
    text: &str,
    key_path: &[&str],
    new_value: &toml::Value,
    tab_size: usize,
) -> Result<(Range<usize>, String)> {
    if let Some((array_node, array_range)) = find_array_value_node(text, key_path) {
        // Finde schließende ']'
        let mut close_bracket = None;
        let mut i = 0;
        while i < array_node.child_count() {
            let c = array_node.child(i).unwrap();
            if c.kind() == "]" {
                close_bracket = Some(c);
                break;
            }
            i += 1;
        }
        let close = close_bracket.ok_or_else(|| anyhow!("Malformed array (missing ])"))?;
        let insert_pos = close.start_byte();

        let mut has_any = false;
        i = 0;
        while i < array_node.child_count() {
            let c = array_node.child(i).unwrap();
            if !matches!(c.kind(), "[" | "]" | "," | "comment") && !c.is_missing() && !c.is_error()
            {
                has_any = true;
                break;
            }
            i += 1;
        }

        let serialized = serialize_toml_value(new_value, tab_size);
        let replacement = if has_any {
            // Erst prüfen ob bereits trailing-Komma/Spacing vorhanden
            let after_previous = &text[array_range.start..insert_pos];
            if after_previous.trim_end().ends_with(',') {
                format!(" {}", serialized)
            } else {
                format!(", {}", serialized)
            }
        } else {
            serialized
        };
        Ok((insert_pos..insert_pos, replacement))
    } else {
        // Array existiert nicht: Erstellen via normalem Key Insert
        let mut new_val = toml::Value::Array(vec![new_value.clone()]);
        let (range, replacement) = ensure_key_path_with_value(text, key_path, &new_val, tab_size);
        Ok((range, replacement))
    }
}

/// Ersetzt / entfernt Eintrag (Index) in Arrays-of-Tables: `[[a.b]]`.
///
/// - `table_path` = Pfad OHNE doppelten Block (z.B. a,b)
/// - `index` = welches Element (0-basiert)
/// - `new_values` = vollständige Tabelle (Map) oder None => Löschen
pub fn replace_array_of_tables_entry_in_toml_text(
    text: &str,
    table_path: &[&str],
    index: usize,
    new_values: Option<&BTreeMap<String, toml::Value>>,
    tab_size: usize,
) -> Result<(Range<usize>, String)> {
    let elements = collect_array_of_tables_elements(text, table_path)?;
    if index >= elements.len() {
        if let Some(values) = new_values {
            // Append
            return append_array_of_tables_entry_in_toml_text(text, table_path, values, tab_size);
        } else {
            return Ok((0..0, String::new()));
        }
    }
    let elem = &elements[index];
    if new_values.is_none() {
        // Ganzes Element löschen + nachfolgende Leerzeilen falls nur whitespace
        let mut end = elem.range.end;
        // Konsumiere nachfolgende blank lines
        if let Some(rest) = text.get(end..) {
            let mut offset = 0;
            for line in rest.split_inclusive('\n') {
                if line.trim().is_empty() {
                    offset += line.len();
                } else {
                    break;
                }
            }
            end += offset;
        }
        return Ok((elem.range.start..end, String::new()));
    }

    // Ersetze kompletten Block-Inhalt (ab Header bis vor nächste Header)
    let mut replacement = format!("[[{}]]\n", table_path.join("."));
    for (k, v) in new_values.unwrap().iter() {
        replacement.push_str(k);
        replacement.push_str(" = ");
        replacement.push_str(&serialize_toml_value(v, tab_size));
        replacement.push('\n');
    }
    Ok((elem.range.clone(), replacement))
}

/// Hängt ein neues `[[path]]` Element an (am Dateiende oder nach letztem Element).
pub fn append_array_of_tables_entry_in_toml_text(
    text: &str,
    table_path: &[&str],
    values: &BTreeMap<String, toml::Value>,
    tab_size: usize,
) -> Result<(Range<usize>, String)> {
    // Sammle existierende Elemente (falls vorhanden)
    let elements = collect_array_of_tables_elements(text, table_path).unwrap_or_default();
    let insert_pos = if let Some(last) = elements.last() {
        last.range.end
    } else {
        // An Dateiende
        text.len()
    };

    let mut repl = String::new();
    if insert_pos > 0 && !text[..insert_pos].ends_with('\n') {
        repl.push('\n');
    }
    repl.push_str("[[");
    repl.push_str(&table_path.join("."));
    repl.push_str("]]\n");
    for (k, v) in values.iter() {
        repl.push_str(k);
        repl.push_str(" = ");
        repl.push_str(&serialize_toml_value(v, tab_size));
        repl.push('\n');
    }
    Ok((insert_pos..insert_pos, repl))
}

/* ------------------------------------------------------------------------------------------------
Interne Kernfunktion analog replace_value_in_json_text
------------------------------------------------------------------------------------------------ */

/// Interner Kern zum (Neu)Setzen, Löschen oder Key-Renaming.
///
/// key_path: vollständiger Pfad (Tabellen + Leaf-Key)
/// new_value: None => Löschen
/// replace_key: Option => Key-Name ersetzen
///
/// Rückgabe (Range, Replacement):
/// - Range, der ersetzt werden soll (kann leer sein bei Insert).
fn replace_value_in_toml_text(
    text: &str,
    key_path: &[&str],
    new_value: Option<&toml::Value>,
    replace_key: Option<&str>,
    tab_size: usize,
) -> (Range<usize>, String) {
    if key_path.is_empty() {
        return (0..0, String::new());
    }

    // Aufteilen in Tabellenpfad & Blatt-Key
    let (table_path, leaf_key) = key_path.split_at(key_path.len() - 1);
    let leaf_key = leaf_key[0];

    let mut parser = Parser::new();
    if let Err(_) = parser.set_language(&tree_sitter_toml::language().into()) {
        return (0..0, String::new());
    }
    let tree = match parser.parse(text, None) {
        Some(t) => t,
        None => return (0..0, String::new()),
    };

    // Tabellen-Block finden
    let table_span = match find_table_block_span(text, &tree, table_path) {
        Ok(r) => r,
        Err(_) => {
            // Tabelle existiert nicht => Insert (falls Wert)
            if let Some(v) = new_value {
                let (range, replacement) = ensure_key_path_with_value(text, key_path, v, tab_size);
                return (range, replacement);
            } else {
                return (0..0, String::new());
            }
        }
    };

    // Key finden im Block
    let kv = find_key_line_and_value_in_span(text, table_span.clone(), leaf_key);
    match kv {
        Some(KeyMatch {
            value_range,
            full_line_range,
            key_range,
            ..
        }) => {
            if new_value.is_none() {
                // Löschen ganze Zeile
                let mut line_end = full_line_range.end;
                // Entferne nachfolgende komplett leere Zeilen (Whitespace)
                if let Some(rest) = text.get(line_end..) {
                    let mut consumed = 0;
                    for l in rest.split_inclusive('\n') {
                        if l.trim().is_empty() {
                            consumed += l.len();
                        } else {
                            break;
                        }
                    }
                    line_end += consumed;
                }
                return (full_line_range.start..line_end, String::new());
            } else {
                let new_val = serialize_toml_value(new_value.unwrap(), tab_size);
                if let Some(new_name) = replace_key {
                    // Komplette Zeile rekonstruieren:
                    // Hol dir die Originalzeile
                    let orig_line = &text[full_line_range.clone()];
                    let (indent, _old_key, after_key) = split_line_prefix_key(
                        orig_line,
                        key_range.start - full_line_range.start,
                        key_range.end - full_line_range.start,
                    );
                    // Versuche inline-comment (ab '#') außerhalb Strings zu detektieren.
                    let comment = extract_inline_comment(orig_line);
                    let line_no_comment = if let Some((c_off, _c)) = comment {
                        &orig_line[..c_off]
                    } else {
                        orig_line
                    };
                    // Versuche '=' Position beizubehalten
                    let eq_pos = line_no_comment.find('=');
                    let reconstructed = if let Some(eq_pos) = eq_pos {
                        let before_eq = &line_no_comment[..eq_pos];
                        let after_eq = &line_no_comment[eq_pos + 1..];
                        // Normalisiere spacing minimal
                        let mut new_line = String::new();
                        new_line.push_str(indent);
                        new_line.push_str(new_name);
                        new_line.push_str(" =");
                        if !after_eq.starts_with(' ') {
                            new_line.push(' ');
                        }
                        new_line.push_str(&new_val);
                        if let Some((_off, ctext)) = comment {
                            new_line.push(' ');
                            new_line.push_str(ctext);
                        }
                        if !new_line.ends_with('\n') {
                            new_line.push('\n');
                        }
                        new_line
                    } else {
                        // Fallback
                        let mut new_line = format!("{indent}{new_name} = {new_val}");
                        if let Some((_off, ctext)) = comment {
                            new_line.push(' ');
                            new_line.push_str(ctext);
                        }
                        new_line.push('\n');
                        new_line
                    };
                    return (full_line_range, reconstructed);
                } else {
                    // Nur Wert ersetzen -> minimal
                    return (value_range, new_val);
                }
            }
        }
        None => {
            if let Some(v) = new_value {
                let (range, replacement) =
                    insert_new_key_in_block(text, table_span, table_path, leaf_key, v, tab_size);
                return (range, replacement);
            } else {
                return (0..0, String::new());
            }
        }
    }
}

/* ------------------------------------------------------------------------------------------------
Arrays-of-Tables Unterstützung
------------------------------------------------------------------------------------------------ */

/// Einzelnes Array-of-tables Element.
#[derive(Debug)]
struct ArrayOfTablesElement {
    range: Range<usize>, // Gesamter Block von [[path]] bis direkt vor nächste Tabelle / EOF
}

/// Sammle alle `[[path]]` Blöcke.
fn collect_array_of_tables_elements(
    text: &str,
    table_path: &[&str],
) -> Result<Vec<ArrayOfTablesElement>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_toml::language().into())
        .context("Failed to set language")?;
    let tree = parser
        .parse(text, None)
        .ok_or_else(|| anyhow!("Parse error"))?;

    let mut qc = QueryCursor::new();
    let matches = qc.matches(&TABLE_HEADER_QUERY, tree.root_node(), text.as_bytes());

    // Wir sammeln alle table_array_element Nodes mit passendem dotted_key
    let mut elements: Vec<Node> = Vec::new();
    for m in matches {
        let mut dotted: Option<Node> = None;
        let mut header_parent: Option<Node> = None;
        for cap in m.captures {
            if cap.node.kind() == "dotted_key" {
                if dotted_key_equals(text, cap.node, table_path) {
                    // Prüfen ob Eltern-Kette table_array_element
                    let maybe_parent = cap.node.parent().and_then(|p| p.parent());
                    if let Some(parent) = maybe_parent {
                        if parent.kind() == "table_array_element" {
                            dotted = Some(cap.node);
                            header_parent = Some(parent);
                        }
                    }
                }
            }
        }
        if let Some(parent) = header_parent {
            elements.push(parent);
        }
    }

    // Ableiten der Block-Ranges (bis vor nächste Tabelle oder EOF)
    let mut results = Vec::new();
    for node in elements {
        let start = node.start_byte();
        let end = next_table_start_or_eof(text, node).unwrap_or(text.len());
        results.push(ArrayOfTablesElement { range: start..end });
    }

    Ok(results)
}

/* ------------------------------------------------------------------------------------------------
Hilfsfunktionen: Tabellen & Keys
------------------------------------------------------------------------------------------------ */

/// Ergebnis eines Key-Matches in einem Tabellenblock.
struct KeyMatch {
    value_range: Range<usize>,
    full_line_range: Range<usize>,
    key_range: Range<usize>,
}

fn find_key_line_and_value_in_span(
    text: &str,
    span: Range<usize>,
    leaf_key: &str,
) -> Option<KeyMatch> {
    let sub = &text[span.clone()];
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_toml::language().into())
        .ok()?;
    let tree = parser.parse(sub, None)?;

    let mut qc = QueryCursor::new();
    let matches = qc.matches(&KEY_VALUE_QUERY, tree.root_node(), sub.as_bytes());
    for m in matches {
        let mut key_node = None;
        let mut value_node = None;
        for cap in m.captures {
            match cap.index {
                0 => key_node = Some(cap.node),
                1 => value_node = Some(cap.node),
                _ => {}
            }
        }
        if let (Some(k), Some(v)) = (key_node, value_node) {
            let raw_key = &sub[k.byte_range()];
            let candidate = extract_last_key_segment(raw_key);
            if candidate == leaf_key {
                let value_range = span.start + v.start_byte()..span.start + v.end_byte();
                let full_line = line_range_covering(
                    text,
                    span.start + k.start_byte(),
                    span.start + v.end_byte(),
                );
                let key_range = span.start + k.start_byte()..span.start + k.end_byte();
                return Some(KeyMatch {
                    value_range,
                    full_line_range: full_line,
                    key_range,
                });
            }
        }
    }
    None
}

fn find_table_block_span(
    text: &str,
    tree: &tree_sitter::Tree,
    table_path: &[&str],
) -> Result<Range<usize>> {
    if table_path.is_empty() {
        // Root = bis zur ersten Tabelle
        let mut cursor = tree.walk();
        cursor.goto_first_child();
        let mut first_table = None;
        loop {
            let node = cursor.node();
            if node.kind() == "table" || node.kind() == "table_array_element" {
                first_table = Some(node.start_byte());
                break;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        return Ok(0..first_table.unwrap_or(text.len()));
    }

    let mut qc = QueryCursor::new();
    let matches = qc.matches(&TABLE_HEADER_QUERY, tree.root_node(), text.as_bytes());
    for m in matches {
        for cap in m.captures {
            if cap.node.kind() == "dotted_key" && dotted_key_equals(text, cap.node, table_path) {
                let table_node = cap
                    .node
                    .parent()
                    .and_then(|h| h.parent())
                    .unwrap_or(cap.node.parent().unwrap());
                let start = table_node.start_byte();
                let end = next_table_start_or_eof(text, table_node)?;
                return Ok(start..end);
            }
        }
    }
    Err(anyhow!("Table {:?} not found", table_path))
}

fn extract_last_key_segment(raw: &str) -> String {
    if raw.contains('.') {
        raw.split('.')
            .last()
            .map(|s| trim_quotes(s.trim()))
            .unwrap_or_default()
    } else {
        trim_quotes(raw.trim())
    }
}

fn trim_quotes(s: &str) -> String {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn line_range_covering(text: &str, start: usize, end: usize) -> Range<usize> {
    let line_start = text[..start].rfind('\n').map_or(0, |i| i + 1);
    let line_end = text[end..].find('\n').map_or(text.len(), |i| end + i + 1);
    line_start..line_end
}

fn dotted_key_equals(text: &str, dotted_key_node: Node, path: &[&str]) -> bool {
    let mut segs = Vec::new();
    let mut i = 0;
    while i < dotted_key_node.child_count() {
        let c = dotted_key_node.child(i).unwrap();
        match c.kind() {
            "bare_key" | "string" => {
                let raw = &text[c.byte_range()];
                segs.push(trim_quotes(raw.trim()));
            }
            _ => {}
        }
        i += 1;
    }
    segs == path
}

fn next_table_start_or_eof(text: &str, table_node: Node) -> Result<usize> {
    let parent = table_node.parent();
    let mut end = text.len();
    if let Some(parent) = parent {
        let mut cursor = parent.walk();
        cursor.goto_first_child();
        let mut seen_self = false;
        loop {
            let n = cursor.node();
            if n.id() == table_node.id() {
                seen_self = true;
            } else if seen_self && (n.kind() == "table" || n.kind() == "table_array_element") {
                end = n.start_byte();
                break;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    Ok(end)
}

/* ------------------------------------------------------------------------------------------------
Einfügen / Erstellen fehlender Tabellen / Keys
------------------------------------------------------------------------------------------------ */

fn ensure_key_path_with_value(
    text: &str,
    full_path: &[&str],
    value: &toml::Value,
    tab_size: usize,
) -> (Range<usize>, String) {
    // Erzeugen neuer Tabellen am Dateiende
    let mut content = String::new();
    if !text.ends_with('\n') {
        content.push('\n');
    }
    if full_path.len() > 1 {
        content.push('[');
        content.push_str(&full_path[..full_path.len() - 1].join("."));
        content.push_str("]\n");
    }
    let leaf = full_path.last().unwrap();
    content.push_str(leaf);
    content.push_str(" = ");
    content.push_str(&serialize_toml_value(value, tab_size));
    content.push('\n');
    (text.len()..text.len(), content)
}

fn insert_new_key_in_block(
    text: &str,
    block_span: Range<usize>,
    table_path: &[&str],
    leaf_key: &str,
    value: &toml::Value,
    tab_size: usize,
) -> (Range<usize>, String) {
    if table_path.is_empty() {
        // Root Insert: ans Ende des Root-Bereichs
        let pos = block_span.end;
        let mut prefix = String::new();
        if pos > 0 && !text[..pos].ends_with('\n') {
            prefix.push('\n');
        }
        let line = format!(
            "{prefix}{leaf_key} = {}\n",
            serialize_toml_value(value, tab_size)
        );
        return (pos..pos, line);
    } else {
        // Existierende Tabelle: ans Ende des Blocks (vor trailing whitespace)
        let block_text = &text[block_span.clone()];
        let mut trimmed_end = block_text.len();
        while trimmed_end > 0 && block_text[..trimmed_end].ends_with(|c: char| c.is_whitespace()) {
            trimmed_end -= 1;
        }
        let insert_pos = block_span.start + trimmed_end;
        let mut prefix = String::new();
        if insert_pos > 0 && !text[..insert_pos].ends_with('\n') {
            prefix.push('\n');
        }
        let line = format!(
            "{prefix}{leaf_key} = {}\n",
            serialize_toml_value(value, tab_size)
        );
        return (insert_pos..insert_pos, line);
    }
}

/* ------------------------------------------------------------------------------------------------
Root-Level Public Convenience (optional)
------------------------------------------------------------------------------------------------ */

/// Convenience: Setzt (Upsert) einen Wert (fully qualified path).
pub fn set_toml_value(
    text: &mut String,
    key_path: &[&str],
    value: &toml::Value,
    tab_size: usize,
) -> Result<TomlEdit> {
    let (range, replacement) =
        replace_value_in_toml_text(text, key_path, Some(value), None, tab_size);
    text.replace_range(range.clone(), &replacement);
    Ok(TomlEdit { range, replacement })
}

/// Convenience: Entfernt Key (falls vorhanden).
pub fn remove_toml_key(
    text: &mut String,
    key_path: &[&str],
    tab_size: usize,
) -> Result<Option<TomlEdit>> {
    let (range, replacement) = replace_value_in_toml_text(text, key_path, None, None, tab_size);
    if range.start == range.end {
        return Ok(None);
    }
    text.replace_range(range.clone(), &replacement);
    Ok(Some(TomlEdit { range, replacement }))
}

/* ------------------------------------------------------------------------------------------------
Value Serialisierung
------------------------------------------------------------------------------------------------ */

fn serialize_toml_value(value: &toml::Value, _indent_size: usize) -> String {
    // toml::Value::to_string() erzeugt gültige Repräsentation.
    // Optional könnte man bei Arrays Multi-Line formatting forcieren.
    value.to_string()
}

/* ------------------------------------------------------------------------------------------------
Array Value Lookup
------------------------------------------------------------------------------------------------ */

fn find_array_value_node(text: &str, key_path: &[&str]) -> Option<(Node<'static>, Range<usize>)> {
    if key_path.is_empty() {
        return None;
    }
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_toml::language().into())
        .ok()?;
    let tree = parser.parse(text, None)?;

    let (table_path, leaf_key) = key_path.split_at(key_path.len() - 1);
    let leaf_key = leaf_key[0];
    let span = find_table_block_span(text, &tree, table_path).ok()?;

    // Parse Subbereich
    let sub = &text[span.clone()];
    let mut sub_parser = Parser::new();
    sub_parser
        .set_language(&tree_sitter_toml::language().into())
        .ok()?;
    let sub_tree = sub_parser.parse(sub, None)?;

    let mut qc = QueryCursor::new();
    let matches = qc.matches(&KEY_VALUE_QUERY, sub_tree.root_node(), sub.as_bytes());
    for m in matches {
        let mut k_node = None;
        let mut v_node = None;
        for cap in m.captures {
            match cap.index {
                0 => k_node = Some(cap.node),
                1 => v_node = Some(cap.node),
                _ => {}
            }
        }
        if let (Some(k), Some(v)) = (k_node, v_node) {
            let raw_key = &sub[k.byte_range()];
            let cand = extract_last_key_segment(raw_key);
            if cand == leaf_key && v.kind() == "array" {
                let global = span.start + v.start_byte()..span.start + v.end_byte();
                return Some((v, global));
            }
        }
    }
    None
}

/* ------------------------------------------------------------------------------------------------
Inline Kommentar / Key Parsing Hilfen
------------------------------------------------------------------------------------------------ */

/// Liefert (indent, original_key_text, rest_nach_key) anhand einer Zeile und Key-Node Offsets.
fn split_line_prefix_key(
    line: &str,
    key_start_rel: usize,
    key_end_rel: usize,
) -> (String, String, String) {
    let indent = &line[..key_start_rel];
    let key_text = &line[key_start_rel..key_end_rel];
    let rest = &line[key_end_rel..];
    (indent.to_string(), key_text.to_string(), rest.to_string())
}

/// Sucht '#' Kommentar (erste # außerhalb String-Literalen).
fn extract_inline_comment(line: &str) -> Option<(usize, &str)> {
    let mut in_squote = false;
    let mut in_dquote = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        match c {
            '\'' if !in_dquote => {
                in_squote = !in_squote;
            }
            '"' if !in_squote => {
                in_dquote = !in_dquote;
            }
            '#' if !in_squote && !in_dquote => {
                return Some((i, &line[i..]));
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/* ------------------------------------------------------------------------------------------------
Tests (optional – can be enriched)
------------------------------------------------------------------------------------------------ */

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    #[test]
    fn test_basic_set_and_rename() {
        let mut text = String::from("title = \"App\"\n");
        let edit = set_toml_value(
            &mut text,
            &["package", "version"],
            &Value::String("1.0.0".into()),
            4,
        )
        .unwrap();
        assert!(text.contains("[package]"));
        assert!(text.contains("version = \"1.0.0\""));
        let renamed =
            rename_key_in_toml_text(&mut text, &["package"], "version", "ver", 4).unwrap();
        assert!(renamed.is_some());
        assert!(text.contains("ver = \"1.0.0\""));
    }

    #[test]
    fn test_array_element_ops() {
        let mut text = String::from("nums = [1, 2, 3]\n");
        let (range, repl) =
            replace_array_element_in_toml_text(&text, &["nums"], 1, Some(&Value::Integer(42)), 4)
                .unwrap();
        text.replace_range(range.clone(), &repl);
        assert_eq!(text, "nums = [1, 42, 3]\n");
        let (range, repl) =
            replace_array_element_in_toml_text(&text, &["nums"], 0, None, 4).unwrap();
        text.replace_range(range.clone(), &repl);
        assert_eq!(text, "nums = [ 42, 3]\n");
    }

    #[test]
    fn test_array_of_tables_append_and_replace() {
        let mut text = String::new();
        let mut map = BTreeMap::new();
        map.insert("id".into(), Value::Integer(1));
        let (_r, repl) =
            append_array_of_tables_entry_in_toml_text(&text, &["item"], &map, 4).unwrap();
        text.push_str(&repl);
        assert!(text.contains("[[item]]"));
        assert!(text.contains("id = 1"));
        // Replace existing element
        let mut map2 = BTreeMap::new();
        map2.insert("id".into(), Value::Integer(99));
        map2.insert("name".into(), Value::String("X".into()));
        let (r2, repl2) =
            replace_array_of_tables_entry_in_toml_text(&text, &["item"], 0, Some(&map2), 4)
                .unwrap();
        text.replace_range(r2, &repl2);
        assert!(text.contains("id = 99"));
        assert!(text.contains("name = \"X\""));
    }
}
