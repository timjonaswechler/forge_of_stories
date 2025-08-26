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

// pub fn update_value_in_json_text<'a>(
//     text: &mut String,
//     key_path: &mut Vec<&'a str>,
//     tab_size: usize,
//     old_value: &'a Value,
//     new_value: &'a Value,
//     preserved_keys: &[&str],
//     edits: &mut Vec<(Range<usize>, String)>,
// ) {
//     // If the old and new values are both objects, then compare them key by key,
//     // preserving the comments and formatting of the unchanged parts. Otherwise,
//     // replace the old value with the new value.
//     // if let (Value::Object(old_object), Value::Object(new_object)) = (old_value, new_value) {
//     //     for (key, old_sub_value) in old_object.iter() {
//     //         key_path.push(key);
//     //         if let Some(new_sub_value) = new_object.get(key) {
//     //             // Key exists in both old and new, recursively update
//     //             update_value_in_json_text(
//     //                 text,
//     //                 key_path,
//     //                 tab_size,
//     //                 old_sub_value,
//     //                 new_sub_value,
//     //                 preserved_keys,
//     //                 edits,
//     //             );
//     //         } else {
//     //             // Key was removed from new object, remove the entire key-value pair
//     //             let (range, replacement) =
//     //                 replace_value_in_json_text(text, key_path, 0, None, None);
//     //             text.replace_range(range.clone(), &replacement);
//     //             edits.push((range, replacement));
//     //         }
//     //         key_path.pop();
//     //     }
//     //     for (key, new_sub_value) in new_object.iter() {
//     //         key_path.push(key);
//     //         if !old_object.contains_key(key) {
//     //             update_value_in_json_text(
//     //                 text,
//     //                 key_path,
//     //                 tab_size,
//     //                 &Value::Null,
//     //                 new_sub_value,
//     //                 preserved_keys,
//     //                 edits,
//     //             );
//     //         }
//     //         key_path.pop();
//     //     }
//     // } else if key_path
//     //     .last()
//     //     .map_or(false, |key| preserved_keys.contains(key))
//     //     || old_value != new_value
//     // {
//     //     let mut new_value = new_value.clone();
//     //     if let Some(new_object) = new_value.as_object_mut() {
//     //         new_object.retain(|_, v| !v.is_null());
//     //     }
//     //     let (range, replacement) =
//     //         replace_value_in_json_text(text, key_path, tab_size, Some(&new_value), None);
//     //     text.replace_range(range.clone(), &replacement);
//     //     edits.push((range, replacement));
//     // }
// }

// /// * `replace_key` - When an exact key match according to `key_path` is found, replace the key with `replace_key` if `Some`.
// fn replace_value_in_json_text(
//     text: &str,
//     key_path: &[&str],
//     tab_size: usize,
//     new_value: Option<&Value>,
//     replace_key: Option<&str>,
// ) -> (Range<usize>, String) {
//     // static PAIR_QUERY: LazyLock<Query> = LazyLock::new(|| {
//     //     Query::new(
//     //         &tree_sitter_json::LANGUAGE.into(),
//     //         "(pair key: (string) @key value: (_) @value)",
//     //     )
//     //     .expect("Failed to create PAIR_QUERY")
//     // });

//     // let mut parser = tree_sitter::Parser::new();
//     // parser
//     //     .set_language(&tree_sitter_json::LANGUAGE.into())
//     //     .unwrap();
//     // let syntax_tree = parser.parse(text, None).unwrap();

//     // let mut cursor = tree_sitter::QueryCursor::new();

//     // let mut depth = 0;
//     // let mut last_value_range = 0..0;
//     // let mut first_key_start = None;
//     // let mut existing_value_range = 0..text.len();

//     // let mut matches = cursor.matches(&PAIR_QUERY, syntax_tree.root_node(), text.as_bytes());
//     // while let Some(mat) = matches.next() {
//     //     if mat.captures.len() != 2 {
//     //         continue;
//     //     }

//     //     let key_range = mat.captures[0].node.byte_range();
//     //     let value_range = mat.captures[1].node.byte_range();

//     //     // Don't enter sub objects until we find an exact
//     //     // match for the current keypath
//     //     if last_value_range.contains_inclusive(&value_range) {
//     //         continue;
//     //     }

//     //     last_value_range = value_range.clone();

//     //     if key_range.start > existing_value_range.end {
//     //         break;
//     //     }

//     //     first_key_start.get_or_insert(key_range.start);

//     //     let found_key = text
//     //         .get(key_range.clone())
//     //         .map(|key_text| {
//     //             depth < key_path.len() && key_text == format!("\"{}\"", key_path[depth])
//     //         })
//     //         .unwrap_or(false);

//     //     if found_key {
//     //         existing_value_range = value_range;
//     //         // Reset last value range when increasing in depth
//     //         last_value_range = existing_value_range.start..existing_value_range.start;
//     //         depth += 1;

//     //         if depth == key_path.len() {
//     //             break;
//     //         }

//     //         first_key_start = None;
//     //     }
//     // }

//     // // We found the exact key we want
//     // if depth == key_path.len() {
//     //     if let Some(new_value) = new_value {
//     //         let new_val = to_pretty_json(new_value, tab_size, tab_size * depth);
//     //         if let Some(replace_key) = replace_key {
//     //             let new_key = format!("\"{}\": ", replace_key);
//     //             if let Some(key_start) = text[..existing_value_range.start].rfind('"') {
//     //                 if let Some(prev_key_start) = text[..key_start].rfind('"') {
//     //                     existing_value_range.start = prev_key_start;
//     //                 } else {
//     //                     existing_value_range.start = key_start;
//     //                 }
//     //             }
//     //             (existing_value_range, new_key + &new_val)
//     //         } else {
//     //             (existing_value_range, new_val)
//     //         }
//     //     } else {
//     //         let mut removal_start = first_key_start.unwrap_or(existing_value_range.start);
//     //         let mut removal_end = existing_value_range.end;

//     //         // Find the actual key position by looking for the key in the pair
//     //         // We need to extend the range to include the key, not just the value
//     //         if let Some(key_start) = text[..existing_value_range.start].rfind('"') {
//     //             if let Some(prev_key_start) = text[..key_start].rfind('"') {
//     //                 removal_start = prev_key_start;
//     //             } else {
//     //                 removal_start = key_start;
//     //             }
//     //         }

//     //         let mut removed_comma = false;
//     //         // Look backward for a preceding comma first
//     //         let preceding_text = text.get(0..removal_start).unwrap_or("");
//     //         if let Some(comma_pos) = preceding_text.rfind(',') {
//     //             // Check if there are only whitespace characters between the comma and our key
//     //             let between_comma_and_key = text.get(comma_pos + 1..removal_start).unwrap_or("");
//     //             if between_comma_and_key.trim().is_empty() {
//     //                 removal_start = comma_pos;
//     //                 removed_comma = true;
//     //             }
//     //         }
//     //         if let Some(remaining_text) = text.get(existing_value_range.end..)
//     //             && !removed_comma
//     //         {
//     //             let mut chars = remaining_text.char_indices();
//     //             while let Some((offset, ch)) = chars.next() {
//     //                 if ch == ',' {
//     //                     removal_end = existing_value_range.end + offset + 1;
//     //                     // Also consume whitespace after the comma
//     //                     while let Some((_, next_ch)) = chars.next() {
//     //                         if next_ch.is_whitespace() {
//     //                             removal_end += next_ch.len_utf8();
//     //                         } else {
//     //                             break;
//     //                         }
//     //                     }
//     //                     break;
//     //                 } else if !ch.is_whitespace() {
//     //                     break;
//     //                 }
//     //             }
//     //         }
//     //         (removal_start..removal_end, String::new())
//     //     }
//     // } else {
//     //     // We have key paths, construct the sub objects
//     //     let new_key = key_path[depth];

//     //     // We don't have the key, construct the nested objects
//     //     let mut new_value =
//     //         serde_json::to_value(new_value.unwrap_or(&serde_json::Value::Null)).unwrap();
//     //     for key in key_path[(depth + 1)..].iter().rev() {
//     //         new_value = serde_json::json!({ key.to_string(): new_value });
//     //     }

//     //     if let Some(first_key_start) = first_key_start {
//     //         let mut row = 0;
//     //         let mut column = 0;
//     //         for (ix, char) in text.char_indices() {
//     //             if ix == first_key_start {
//     //                 break;
//     //             }
//     //             if char == '\n' {
//     //                 row += 1;
//     //                 column = 0;
//     //             } else {
//     //                 column += char.len_utf8();
//     //             }
//     //         }

//     //         if row > 0 {
//     //             // depth is 0 based, but division needs to be 1 based.
//     //             let new_val = to_pretty_json(&new_value, column / (depth + 1), column);
//     //             let space = ' ';
//     //             let content = format!("\"{new_key}\": {new_val},\n{space:width$}", width = column);
//     //             (first_key_start..first_key_start, content)
//     //         } else {
//     //             let new_val = serde_json::to_string(&new_value).unwrap();
//     //             let mut content = format!(r#""{new_key}": {new_val},"#);
//     //             content.push(' ');
//     //             (first_key_start..first_key_start, content)
//     //         }
//     //     } else {
//     //         new_value = serde_json::json!({ new_key.to_string(): new_value });
//     //         let indent_prefix_len = 4 * depth;
//     //         let mut new_val = to_pretty_json(&new_value, 4, indent_prefix_len);
//     //         if depth == 0 {
//     //             new_val.push('\n');
//     //         }
//     //         // best effort to keep comments with best effort indentation
//     //         let mut replace_text = &text[existing_value_range.clone()];
//     //         while let Some(comment_start) = replace_text.rfind("//") {
//     //             if let Some(comment_end) = replace_text[comment_start..].find('\n') {
//     //                 let mut comment_with_indent_start = replace_text[..comment_start]
//     //                     .rfind('\n')
//     //                     .unwrap_or(comment_start);
//     //                 if !replace_text[comment_with_indent_start..comment_start]
//     //                     .trim()
//     //                     .is_empty()
//     //                 {
//     //                     comment_with_indent_start = comment_start;
//     //                 }
//     //                 new_val.insert_str(
//     //                     1,
//     //                     &replace_text[comment_with_indent_start..comment_start + comment_end],
//     //                 );
//     //             }
//     //             replace_text = &replace_text[..comment_start];
//     //         }

//     //         (existing_value_range, new_val)
//     //     }
//     // }
// }

// const TS_DOCUMENT_KIND: &'static str = "document";
// const TS_ARRAY_KIND: &'static str = "array";
// const TS_COMMENT_KIND: &'static str = "comment";

// pub fn replace_top_level_array_value_in_json_text(
//     text: &str,
//     key_path: &[&str],
//     new_value: Option<&Value>,
//     replace_key: Option<&str>,
//     array_index: usize,
//     tab_size: usize,
// ) -> Result<(Range<usize>, String)> {
//     // let mut parser = tree_sitter::Parser::new();
//     // parser
//     //     .set_language(&tree_sitter_json::LANGUAGE.into())
//     //     .unwrap();
//     // let syntax_tree = parser.parse(text, None).unwrap();

//     // let mut cursor = syntax_tree.walk();

//     // if cursor.node().kind() == TS_DOCUMENT_KIND {
//     //     anyhow::ensure!(
//     //         cursor.goto_first_child(),
//     //         "Document empty - No top level array"
//     //     );
//     // }

//     // while cursor.node().kind() != TS_ARRAY_KIND {
//     //     anyhow::ensure!(cursor.goto_next_sibling(), "EOF - No top level array");
//     // }

//     // // false if no children
//     // //
//     // cursor.goto_first_child();
//     // debug_assert_eq!(cursor.node().kind(), "[");

//     // let mut index = 0;

//     // while index <= array_index {
//     //     let node = cursor.node();
//     //     if !matches!(node.kind(), "[" | "]" | TS_COMMENT_KIND | ",")
//     //         && !node.is_extra()
//     //         && !node.is_missing()
//     //     {
//     //         if index == array_index {
//     //             break;
//     //         }
//     //         index += 1;
//     //     }
//     //     if !cursor.goto_next_sibling() {
//     //         if let Some(new_value) = new_value {
//     //             return append_top_level_array_value_in_json_text(text, new_value, tab_size);
//     //         } else {
//     //             return Ok((0..0, String::new()));
//     //         }
//     //     }
//     // }

//     // let range = cursor.node().range();
//     // let indent_width = range.start_point.column;
//     // let offset = range.start_byte;
//     // let text_range = range.start_byte..range.end_byte;
//     // let value_str = &text[text_range.clone()];
//     // let needs_indent = range.start_point.row > 0;

//     // if new_value.is_none() && key_path.is_empty() {
//     //     let mut remove_range = text_range.clone();
//     //     if index == 0 {
//     //         while cursor.goto_next_sibling()
//     //             && (cursor.node().is_extra() || cursor.node().is_missing())
//     //         {}
//     //         if cursor.node().kind() == "," {
//     //             remove_range.end = cursor.node().range().end_byte;
//     //         }
//     //         if let Some(next_newline) = &text[remove_range.end + 1..].find('\n') {
//     //             if text[remove_range.end + 1..remove_range.end + next_newline]
//     //                 .chars()
//     //                 .all(|c| c.is_ascii_whitespace())
//     //             {
//     //                 remove_range.end = remove_range.end + next_newline;
//     //             }
//     //         }
//     //     } else {
//     //         while cursor.goto_previous_sibling()
//     //             && (cursor.node().is_extra() || cursor.node().is_missing())
//     //         {}
//     //         if cursor.node().kind() == "," {
//     //             remove_range.start = cursor.node().range().start_byte;
//     //         }
//     //     }
//     //     return Ok((remove_range, String::new()));
//     // } else {
//     //     let (mut replace_range, mut replace_value) =
//     //         replace_value_in_json_text(value_str, key_path, tab_size, new_value, replace_key);

//     //     replace_range.start += offset;
//     //     replace_range.end += offset;

//     //     if needs_indent {
//     //         let increased_indent = format!("\n{space:width$}", space = ' ', width = indent_width);
//     //         replace_value = replace_value.replace('\n', &increased_indent);
//     //         // replace_value.push('\n');
//     //     } else {
//     //         while let Some(idx) = replace_value.find("\n ") {
//     //             replace_value.remove(idx + 1);
//     //         }
//     //         while let Some(idx) = replace_value.find("\n") {
//     //             replace_value.replace_range(idx..idx + 1, " ");
//     //         }
//     //     }

//     //     return Ok((replace_range, replace_value));
//     // }
// }

// pub fn append_top_level_array_value_in_json_text(
//     text: &str,
//     new_value: &Value,
//     tab_size: usize,
// ) -> Result<(Range<usize>, String)> {
//     // let mut parser = tree_sitter::Parser::new();
//     // parser
//     //     .set_language(&tree_sitter_json::LANGUAGE.into())
//     //     .unwrap();
//     // let syntax_tree = parser.parse(text, None).unwrap();

//     // let mut cursor = syntax_tree.walk();

//     // if cursor.node().kind() == TS_DOCUMENT_KIND {
//     //     anyhow::ensure!(
//     //         cursor.goto_first_child(),
//     //         "Document empty - No top level array"
//     //     );
//     // }

//     // while cursor.node().kind() != TS_ARRAY_KIND {
//     //     anyhow::ensure!(cursor.goto_next_sibling(), "EOF - No top level array");
//     // }

//     // anyhow::ensure!(
//     //     cursor.goto_last_child(),
//     //     "Malformed JSON syntax tree, expected `]` at end of array"
//     // );
//     // debug_assert_eq!(cursor.node().kind(), "]");
//     // let close_bracket_start = cursor.node().start_byte();
//     // while cursor.goto_previous_sibling()
//     //     && (cursor.node().is_extra() || cursor.node().is_missing())
//     //     && !cursor.node().is_error()
//     // {}

//     // let mut comma_range = None;
//     // let mut prev_item_range = None;

//     // if cursor.node().kind() == "," || is_error_of_kind(&mut cursor, ",") {
//     //     comma_range = Some(cursor.node().byte_range());
//     //     while cursor.goto_previous_sibling()
//     //         && (cursor.node().is_extra() || cursor.node().is_missing())
//     //     {}

//     //     debug_assert_ne!(cursor.node().kind(), "[");
//     //     prev_item_range = Some(cursor.node().range());
//     // } else {
//     //     while (cursor.node().is_extra() || cursor.node().is_missing())
//     //         && cursor.goto_previous_sibling()
//     //     {}
//     //     if cursor.node().kind() != "[" {
//     //         prev_item_range = Some(cursor.node().range());
//     //     }
//     // }

//     // let (mut replace_range, mut replace_value) =
//     //     replace_value_in_json_text("", &[], tab_size, Some(new_value), None);

//     // replace_range.start = close_bracket_start;
//     // replace_range.end = close_bracket_start;

//     // let space = ' ';
//     // if let Some(prev_item_range) = prev_item_range {
//     //     let needs_newline = prev_item_range.start_point.row > 0;
//     //     let indent_width = text[..prev_item_range.start_byte].rfind('\n').map_or(
//     //         prev_item_range.start_point.column,
//     //         |idx| {
//     //             prev_item_range.start_point.column
//     //                 - text[idx + 1..prev_item_range.start_byte].trim_start().len()
//     //         },
//     //     );

//     //     let prev_item_end = comma_range
//     //         .as_ref()
//     //         .map_or(prev_item_range.end_byte, |range| range.end);
//     //     if text[prev_item_end..replace_range.start].trim().is_empty() {
//     //         replace_range.start = prev_item_end;
//     //     }

//     //     if needs_newline {
//     //         let increased_indent = format!("\n{space:width$}", width = indent_width);
//     //         replace_value = replace_value.replace('\n', &increased_indent);
//     //         replace_value.push('\n');
//     //         replace_value.insert_str(0, &format!("\n{space:width$}", width = indent_width));
//     //     } else {
//     //         while let Some(idx) = replace_value.find("\n ") {
//     //             replace_value.remove(idx + 1);
//     //         }
//     //         while let Some(idx) = replace_value.find('\n') {
//     //             replace_value.replace_range(idx..idx + 1, " ");
//     //         }
//     //         replace_value.insert(0, ' ');
//     //     }

//     //     if comma_range.is_none() {
//     //         replace_value.insert(0, ',');
//     //     }
//     // } else {
//     //     if let Some(prev_newline) = text[..replace_range.start].rfind('\n') {
//     //         if text[prev_newline..replace_range.start].trim().is_empty() {
//     //             replace_range.start = prev_newline;
//     //         }
//     //     }
//     //     let indent = format!("\n{space:width$}", width = tab_size);
//     //     replace_value = replace_value.replace('\n', &indent);
//     //     replace_value.insert_str(0, &indent);
//     //     replace_value.push('\n');
//     // }
//     // return Ok((replace_range, replace_value));

//     // fn is_error_of_kind(cursor: &mut tree_sitter::TreeCursor<'_>, kind: &str) -> bool {
//     //     if cursor.node().kind() != "ERROR" {
//     //         return false;
//     //     }

//     //     let descendant_index = cursor.descendant_index();
//     //     let res = cursor.goto_first_child() && cursor.node().kind() == kind;
//     //     cursor.goto_descendant(descendant_index);
//     //     return res;
//     // }
// }
