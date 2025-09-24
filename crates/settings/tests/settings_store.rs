//! Integration tests for the SettingsStore:
//! - Recursive diffing (nested structs)
//! - Persisting only changed (delta) fields
//! - Reloading after external file modification
//!
//! NOTE: These tests avoid adding extra dev-dependencies by using std only.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
#[cfg(feature = "watch")]
use std::time::Duration;

use settings::{Settings, SettingsStore};

use serde::{Deserialize, Serialize};

fn unique_temp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("settings_store_test_{name}_{nanos}.ron"));
    p
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Nested {
    enabled: bool,
    level: u8,
}

impl Default for Nested {
    fn default() -> Self {
        Self {
            enabled: false,
            level: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Network {
    port: u16,
    nested: Nested,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            port: 100,
            nested: Nested::default(),
        }
    }
}

impl Settings for Network {
    const SECTION: &'static str = "network";
}

#[test]
fn register_get_update_delta_flat_and_nested() {
    let path = unique_temp_path("delta_flat_nested");
    // Ensure clean
    let _ = fs::remove_file(&path);

    let store = SettingsStore::builder()
        .with_settings_file(path.clone())
        .build()
        .expect("build store");

    // Register
    store.register::<Network>().expect("register network");

    // File should not exist yet (no changes persisted)
    assert!(
        !path.exists(),
        "Expected no file before first update, found: {path:?}"
    );

    // Initial get
    let net = store.get::<Network>().expect("get initial");
    assert_eq!(net.port, 100);
    assert_eq!(net.nested.enabled, false);
    assert_eq!(net.nested.level, 1);

    // Update ONLY nested.enabled -> true (port unchanged, nested.level unchanged)
    store
        .update::<Network, _>(|n| {
            n.nested.enabled = true;
        })
        .expect("update nested.enabled");

    // File must exist now
    assert!(path.exists(), "Delta file should be created");

    let content = fs::read_to_string(&path).expect("read delta file");
    // Parse as generic structure
    let root: std::collections::HashMap<String, ron::Value> =
        ron::from_str(&content).expect("parse delta RON");

    // Only "network" key at root
    assert!(root.contains_key("network"));

    let network_delta = root.get("network").unwrap();
    let ron::Value::Map(section_map) = network_delta else {
        panic!("network entry should be a map");
    };

    // Convert keys to string set
    let mut top_keys = HashSet::new();
    for (k, _v) in section_map.iter() {
        if let ron::Value::String(s) = k {
            top_keys.insert(s.clone());
        }
    }

    // We changed no top-level simple field => "port" MUST NOT appear
    assert!(
        !top_keys.contains("port"),
        "port should not be in delta (unchanged)"
    );
    assert!(
        top_keys.contains("nested"),
        "nested should appear because nested.enabled changed"
    );

    // Inspect nested map delta
    let nested_value = section_map
        .iter()
        .find(|(k, _)| matches!(k, ron::Value::String(s) if s == "nested"))
        .map(|(_, v)| v)
        .unwrap();
    let ron::Value::Map(nested_map) = nested_value else {
        panic!("nested delta should be a map");
    };
    let mut nested_keys = HashSet::new();
    for (k, _v) in nested_map.iter() {
        if let ron::Value::String(s) = k {
            nested_keys.insert(s.clone());
        }
    }
    assert!(
        nested_keys.contains("enabled"),
        "nested.enabled was changed so must be present"
    );
    assert!(
        !nested_keys.contains("level"),
        "nested.level is unchanged so must not appear"
    );

    // Now perform an update changing top-level port AND nested.level
    store
        .update::<Network, _>(|n| {
            n.port = 7777;
            n.nested.level = 5;
        })
        .expect("update port + nested.level");

    let content2 = fs::read_to_string(&path).expect("read second delta");
    let root2: std::collections::HashMap<String, ron::Value> =
        ron::from_str(&content2).expect("parse second delta");
    let network_entry = root2.get("network").unwrap();
    let ron::Value::Map(section_map2) = network_entry else {
        panic!("network entry should be a map");
    };

    let mut top_keys2 = HashSet::new();
    for (k, _v) in section_map2.iter() {
        if let ron::Value::String(s) = k {
            top_keys2.insert(s.clone());
        }
    }
    assert!(top_keys2.contains("port"), "port changed => must appear");
    assert!(top_keys2.contains("nested"), "nested still in delta");

    let nested_value2 = section_map2
        .iter()
        .find(|(k, _)| matches!(k, ron::Value::String(s) if s == "nested"))
        .map(|(_, v)| v)
        .unwrap();
    let ron::Value::Map(nested_map2) = nested_value2 else {
        panic!("nested delta should be map");
    };
    let mut nested_keys2 = HashSet::new();
    for (k, _v) in nested_map2.iter() {
        if let ron::Value::String(s) = k {
            nested_keys2.insert(s.clone());
        }
    }
    // After second update, enabled was already diverged (remains in diff),
    // level changed now -> must appear too.
    assert!(nested_keys2.contains("enabled"));
    assert!(nested_keys2.contains("level"));

    // Sanity: effective value
    let net2 = store.get::<Network>().expect("get after second update");
    assert_eq!(net2.port, 7777);
    assert_eq!(net2.nested.level, 5);
    assert_eq!(net2.nested.enabled, true);
}

#[test]
fn reload_applies_external_changes() {
    let path = unique_temp_path("reload");
    let _ = fs::remove_file(&path);

    let store = SettingsStore::builder()
        .with_settings_file(path.clone())
        .build()
        .expect("build");
    store.register::<Network>().expect("register");

    // Make an initial change
    store
        .update::<Network, _>(|n| n.port = 1500)
        .expect("initial update");

    // Simulate external modification: overwrite file with new delta (port=9000)
    let external = r#"
    {
        "network": {
            "port": 9000,
            "nested": { "enabled": true }
        }
    }
    "#;
    fs::write(&path, external).expect("write external delta");

    // Reload
    store.reload().expect("reload after external change");

    let net = store.get::<Network>().expect("get after reload");
    assert_eq!(net.port, 9000);
    assert_eq!(net.nested.enabled, true);
    assert_eq!(
        net.nested.level, 1,
        "unchanged nested.level should remain default"
    );
}

#[cfg(feature = "watch")]
#[test]
fn watcher_triggers_reload() {
    use std::sync::Arc;
    use std::thread;

    let path = unique_temp_path("watch");
    let _ = fs::remove_file(&path);

    let store = Arc::new(
        SettingsStore::builder()
            .with_settings_file(path.clone())
            .build()
            .expect("build"),
    );
    store.register::<Network>().expect("register");

    let _watcher = settings::start_settings_watcher(store.clone()).expect("start watcher");

    // External change
    let external = r#"
    {
        "network": {
            "port": 4242
        }
    }
    "#;
    fs::write(&path, external).expect("write external delta");

    // Give the watcher some time (depends on backend, keep short but non-zero)
    thread::sleep(Duration::from_millis(250));

    let net = store.get::<Network>().expect("get after watch reload");
    assert_eq!(net.port, 4242);
}
