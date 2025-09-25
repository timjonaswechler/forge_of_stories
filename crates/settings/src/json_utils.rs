#![allow(dead_code)]

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{ops::Range, str, sync::LazyLock};
use tree_sitter::Query;

/// Update a JSON text in-place while preserving formatting and comments.
///
/// Ported from Zed's `settings_json.rs` utility.
pub(crate) fn update_value_in_json_text<'a>(
    text: &mut String,
    key_path: &mut Vec<&'a str>,
    tab_size: usize,
    old_value: &'a Value,
    new_value: &'a Value,
    edits: &mut Vec<(Range<usize>, String)>,
) {
    if let (Value::Object(old_object), Value::Object(new_object)) = (old_value, new_value) {
        for (key, old_sub_value) in old_object.iter() {
            key_path.push(key);
            if let Some(new_sub_value) = new_object.get(key) {
                update_value_in_json_text(
                    text,
                    key_path,
                    tab_size,
                    old_sub_value,
                    new_sub_value,
                    edits,
                );
            } else {
                let (range, replacement) =
                    replace_value_in_json_text(text, key_path, 0, None, None);
                text.replace_range(range.clone(), &replacement);
                edits.push((range, replacement));
            }
            key_path.pop();
        }
        for (key, new_sub_value) in new_object.iter() {
            key_path.push(key);
            if !old_object.contains_key(key) {
                update_value_in_json_text(
                    text,
                    key_path,
                    tab_size,
                    &Value::Null,
                    new_sub_value,
                    edits,
                );
            }
            key_path.pop();
        }
    } else if old_value != new_value {
        let mut new_value = new_value.clone();
        if let Some(new_object) = new_value.as_object_mut() {
            new_object.retain(|_, v| !v.is_null());
        }
        let (range, replacement) =
            replace_value_in_json_text(text, key_path, tab_size, Some(&new_value), None);
        text.replace_range(range.clone(), &replacement);
        edits.push((range, replacement));
    }
}

/// Replace a value at `key_path` inside `text`.
pub(crate) fn replace_value_in_json_text<T: AsRef<str>>(
    text: &str,
    key_path: &[T],
    tab_size: usize,
    new_value: Option<&Value>,
    replace_key: Option<&str>,
) -> (Range<usize>, String) {
    static PAIR_QUERY: LazyLock<Query> = LazyLock::new(|| {
        Query::new(
            tree_sitter_json::language(),
            "(pair key: (string) @key value: (_) @value)",
        )
        .expect("Failed to create PAIR_QUERY")
    });

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_json::language()).unwrap();
    let syntax_tree = parser.parse(text, None).unwrap();

    let mut cursor = tree_sitter::QueryCursor::new();

    let mut depth = 0;
    let mut last_value_range = 0..0;
    let mut first_key_start = None;
    let mut existing_value_range = 0..text.len();

    let mut matches = cursor.matches(&PAIR_QUERY, syntax_tree.root_node(), text.as_bytes());
    while let Some(mat) = matches.next() {
        if mat.captures.len() != 2 {
            continue;
        }

        let key_range = mat.captures[0].node.byte_range();
        let value_range = mat.captures[1].node.byte_range();

        if contains_inclusive(&last_value_range, &value_range) {
            continue;
        }

        last_value_range = value_range.clone();

        if key_range.start > existing_value_range.end {
            break;
        }

        first_key_start.get_or_insert(key_range.start);

        let found_key = text
            .get(key_range.clone())
            .zip(key_path.get(depth))
            .and_then(|(key_text, key_path_value)| {
                serde_json::to_string(key_path_value.as_ref())
                    .ok()
                    .map(|key_path| depth < key_path.len() && key_text == key_path)
            })
            .unwrap_or(false);

        if found_key {
            existing_value_range = value_range;
            last_value_range = existing_value_range.start..existing_value_range.start;
            depth += 1;

            if depth == key_path.len() {
                break;
            }

            if let Some(array_replacement) = handle_possible_array_value(
                &mat.captures[0].node,
                &mat.captures[1].node,
                text,
                &key_path[depth..],
                new_value,
                replace_key,
                tab_size,
            ) {
                return array_replacement;
            }

            first_key_start = None;
        }
    }

    if depth == key_path.len() {
        if let Some(new_value) = new_value {
            let new_val = to_pretty_json(new_value, tab_size, tab_size * depth);
            if let Some(replace_key) = replace_key.and_then(|s| serde_json::to_string(s).ok()) {
                let new_key = format!("{}: ", replace_key);
                if let Some(key_start) = text[..existing_value_range.start].rfind('"') {
                    if let Some(prev_key_start) = text[..key_start].rfind('"') {
                        existing_value_range.start = prev_key_start;
                    } else {
                        existing_value_range.start = key_start;
                    }
                }
                (existing_value_range, new_key + &new_val)
            } else {
                (existing_value_range, new_val)
            }
        } else {
            let mut removal_start = first_key_start.unwrap_or(existing_value_range.start);
            let mut removal_end = existing_value_range.end;

            if let Some(key_start) = text[..existing_value_range.start].rfind('"') {
                if let Some(prev_key_start) = text[..key_start].rfind('"') {
                    removal_start = prev_key_start;
                } else {
                    removal_start = key_start;
                }
            }

            let mut removed_comma = false;
            let preceding_text = text.get(0..removal_start).unwrap_or("");
            if let Some(comma_pos) = preceding_text.rfind(',') {
                let between = text.get(comma_pos + 1..removal_start).unwrap_or("");
                if between.trim().is_empty() {
                    removal_start = comma_pos;
                    removed_comma = true;
                }
            }

            if !removed_comma {
                let following_text = text.get(removal_end..).unwrap_or("");
                if let Some(comma_pos) = following_text.find(',') {
                    let between = &following_text[..comma_pos];
                    if between.trim().is_empty() {
                        removal_end += comma_pos + 1;
                        removed_comma = true;
                    }
                }
            }

            if !removed_comma {
                let preceding_text = text.get(0..removal_start).unwrap_or("");
                if let Some(newline_pos) = preceding_text.rfind('\n') {
                    if preceding_text[newline_pos..].trim().is_empty() {
                        removal_start = newline_pos;
                    }
                }
            }

            let replacement = String::new();
            if !removed_comma {
                if let Some(next_newline) = text[removal_end..].find('\n') {
                    if text[removal_end..removal_end + next_newline]
                        .chars()
                        .all(|c| c.is_ascii_whitespace())
                    {
                        removal_end += next_newline;
                    }
                }
            }

            (removal_start..removal_end, replacement)
        }
    } else {
        let json_value = construct_json_value(key_path, new_value);
        let json_value = serde_json::json!([json_value]);
        (0..text.len(), to_pretty_json(&json_value, tab_size, 0))
    }
}

fn construct_json_value(
    key_path: &[impl AsRef<str>],
    new_value: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut new_value =
        serde_json::to_value(new_value.unwrap_or(&serde_json::Value::Null)).unwrap();
    for key in key_path.iter().rev() {
        if parse_index_key(key.as_ref()).is_some() {
            new_value = serde_json::json!([new_value]);
        } else {
            new_value = serde_json::json!({ key.as_ref().to_string(): new_value });
        }
    }
    new_value
}

fn handle_possible_array_value(
    key_node: &tree_sitter::Node,
    value_node: &tree_sitter::Node,
    text: &str,
    remaining_key_path: &[impl AsRef<str>],
    new_value: Option<&Value>,
    replace_key: Option<&str>,
    tab_size: usize,
) -> Option<(Range<usize>, String)> {
    if remaining_key_path.is_empty() {
        return None;
    }
    let key_path = remaining_key_path;
    let index = parse_index_key(key_path[0].as_ref())?;

    let value_is_array = value_node.kind() == TS_ARRAY_KIND;

    let array_str = if value_is_array {
        &text[value_node.byte_range()]
    } else {
        ""
    };

    let (mut replace_range, mut replace_value) = replace_top_level_array_value_in_json_text(
        array_str,
        &key_path[1..],
        new_value,
        replace_key,
        index,
        tab_size,
    );

    if value_is_array {
        replace_range.start += value_node.start_byte();
        replace_range.end += value_node.start_byte();
    } else {
        replace_range = value_node.byte_range();
    }

    let non_whitespace = replace_value
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .count();
    let needs_indent = replace_value.ends_with('\n')
        || replace_value
            .chars()
            .zip(replace_value.chars().skip(1))
            .any(|(c, next_c)| c == '\n' && !next_c.is_ascii_whitespace());
    let contains_comment = (replace_value.contains("//") && replace_value.contains('\n'))
        || (replace_value.contains("/*") && replace_value.contains("*/"));
    if needs_indent {
        let indent_width = key_node.start_position().column;
        let increased_indent = format!("\n{space:width$}", space = ' ', width = indent_width);
        replace_value = replace_value.replace('\n', &increased_indent);
    } else if non_whitespace < 32 && !contains_comment {
        while let Some(idx) = replace_value.find("\n ") {
            replace_value.remove(idx);
        }
        while let Some(idx) = replace_value.find("  ") {
            replace_value.remove(idx);
        }
    }
    Some((replace_range, replace_value))
}

const TS_DOCUMENT_KIND: &str = "document";
const TS_ARRAY_KIND: &str = "array";

pub(crate) fn replace_top_level_array_value_in_json_text(
    text: &str,
    key_path: &[impl AsRef<str>],
    new_value: Option<&Value>,
    replace_key: Option<&str>,
    array_index: usize,
    tab_size: usize,
) -> (Range<usize>, String) {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_json::language()).unwrap();

    let syntax_tree = parser.parse(text, None).unwrap();

    let mut cursor = syntax_tree.walk();

    if cursor.node().kind() == TS_DOCUMENT_KIND {
        cursor.goto_first_child();
    }

    while cursor.node().kind() != TS_ARRAY_KIND {
        if !cursor.goto_next_sibling() {
            let json_value = construct_json_value(key_path, new_value);
            let json_value = serde_json::json!([json_value]);
            return (0..text.len(), to_pretty_json(&json_value, tab_size, 0));
        }
    }

    cursor.goto_first_child();

    let mut current_index = 0;
    let mut current_node = cursor.node();
    let mut replace_range = current_node.byte_range();
    let mut replace_value = String::new();

    loop {
        if current_node.kind() == TS_ARRAY_KIND {
            cursor.goto_first_child();
            current_node = cursor.node();
            continue;
        }

        if current_node.kind() == "]" {
            break;
        }

        if current_node.is_missing() || current_node.is_extra() {
            if !cursor.goto_next_sibling() {
                break;
            }
            current_node = cursor.node();
            continue;
        }

        if current_index == array_index {
            replace_range = current_node.byte_range();
            break;
        }

        current_index += 1;
        if !cursor.goto_next_sibling() {
            break;
        }
        current_node = cursor.node();
    }

    if current_index != array_index {
        let json_value = construct_json_value(key_path, new_value);
        replace_value = to_pretty_json(&json_value, tab_size, tab_size);
        if !text.trim().ends_with('[') {
            replace_value.insert(0, ',');
        }
        return (text.len()..text.len(), replace_value);
    }

    if let Some(new_value) = new_value {
        replace_value = to_pretty_json(new_value, tab_size, tab_size);
        if let Some(replace_key) = replace_key.and_then(|s| serde_json::to_string(s).ok()) {
            replace_value = format!("{}: {}", replace_key, replace_value);
        }
    }

    if replace_value.contains('\n') || text.contains('\n') {
        if let Some(prev_newline) = text[..replace_range.start].rfind('\n')
            && text[prev_newline..replace_range.start].trim().is_empty()
        {
            replace_range.start = prev_newline;
        }
        let indent = format!("\n{space:width$}", space = ' ', width = tab_size);
        replace_value = replace_value.replace('\n', &indent);
        replace_value.insert_str(0, &indent);
        replace_value.push('\n');
    }

    (replace_range, replace_value)
}

pub(crate) fn append_top_level_array_value_in_json_text(
    text: &str,
    new_value: &Value,
    tab_size: usize,
) -> (Range<usize>, String) {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_json::language()).unwrap();
    let syntax_tree = parser.parse(text, None).unwrap();

    let mut cursor = syntax_tree.walk();

    if cursor.node().kind() == TS_DOCUMENT_KIND {
        cursor.goto_first_child();
    }

    while cursor.node().kind() != TS_ARRAY_KIND {
        if !cursor.goto_next_sibling() {
            let json_value = serde_json::json!([new_value]);
            return (0..text.len(), to_pretty_json(&json_value, tab_size, 0));
        }
    }

    let went_to_last_child = goto_last_child(&mut cursor);
    debug_assert!(went_to_last_child && cursor.node().kind() == "]");
    let close_bracket_start = cursor.node().start_byte();
    while goto_previous_sibling(&mut cursor)
        && (cursor.node().is_extra() || cursor.node().is_missing())
        && !cursor.node().is_error()
    {}

    let mut comma_range = None;
    let mut prev_item_range = None;

    if cursor.node().kind() == "," || is_error_of_kind(&cursor, ",") {
        comma_range = Some(cursor.node().byte_range());
        while goto_previous_sibling(&mut cursor)
            && (cursor.node().is_extra() || cursor.node().is_missing())
        {}

        prev_item_range = Some(cursor.node().byte_range());
    } else {
        while (cursor.node().is_extra() || cursor.node().is_missing())
            && goto_previous_sibling(&mut cursor)
        {}
        if cursor.node().kind() != "[" {
            prev_item_range = Some(cursor.node().byte_range());
        }
    }

    let mut replace_range = close_bracket_start..close_bracket_start;
    let mut replace_value = to_pretty_json(new_value, tab_size, tab_size);

    if let Some(prev_item_range) = prev_item_range {
        if prev_item_range.end != close_bracket_start {
            if let Some(comma_range) = comma_range {
                replace_range = comma_range.start..close_bracket_start;
                replace_value.insert(0, ',');
            } else {
                replace_range = prev_item_range.end..close_bracket_start;
                replace_value.insert(0, ',');
            }
        }
    }

    if replace_value.contains('\n') || text.contains('\n') {
        if let Some(prev_newline) = text[..replace_range.start].rfind('\n')
            && text[prev_newline..replace_range.start].trim().is_empty()
        {
            replace_range.start = prev_newline;
        }
        let indent = format!("\n{space:width$}", space = ' ', width = tab_size);
        replace_value = replace_value.replace('\n', &indent);
        replace_value.insert_str(0, &indent);
        replace_value.push('\n');
    }

    (replace_range, replace_value)
}

fn goto_last_child(cursor: &mut tree_sitter::TreeCursor<'_>) -> bool {
    if !cursor.goto_first_child() {
        return false;
    }

    while cursor.goto_next_sibling() {}
    true
}

fn goto_previous_sibling(cursor: &mut tree_sitter::TreeCursor<'_>) -> bool {
    if let Some(prev) = cursor.node().prev_sibling() {
        *cursor = prev.walk();
        true
    } else {
        false
    }
}

pub(crate) fn to_pretty_json(
    value: &impl Serialize,
    indent_size: usize,
    indent_prefix_len: usize,
) -> String {
    const SPACES: [u8; 32] = [b' '; 32];

    debug_assert!(indent_size <= SPACES.len());
    debug_assert!(indent_prefix_len <= SPACES.len());

    let mut output = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(
        &mut output,
        serde_json::ser::PrettyFormatter::with_indent(&SPACES[0..indent_size.min(SPACES.len())]),
    );

    value.serialize(&mut ser).unwrap();
    let text = String::from_utf8(output).unwrap();

    let mut adjusted_text = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            adjusted_text.push_str(str::from_utf8(&SPACES[0..indent_prefix_len]).unwrap());
        }
        adjusted_text.push_str(line);
        adjusted_text.push('\n');
    }
    adjusted_text.pop();
    adjusted_text
}

pub(crate) fn parse_json_with_comments<T: DeserializeOwned>(content: &str) -> Result<T> {
    let mut deserializer = serde_json_lenient::Deserializer::from_str(content);
    Ok(serde_path_to_error::deserialize(&mut deserializer)?)
}

fn parse_index_key(index_key: &str) -> Option<usize> {
    index_key.strip_prefix('#')?.parse().ok()
}

fn contains_inclusive(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    outer.start <= inner.start && outer.end >= inner.end
}

fn is_error_of_kind(cursor: &tree_sitter::TreeCursor<'_>, kind: &str) -> bool {
    if cursor.node().kind() != "ERROR" {
        return false;
    }

    cursor
        .node()
        .child(0)
        .map(|child| child.kind() == kind)
        .unwrap_or(false)
}
