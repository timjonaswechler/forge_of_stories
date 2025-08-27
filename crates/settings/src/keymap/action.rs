use serde::Deserialize;
use toml_edit::Value;

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(transparent)]
pub struct KeymapAction(pub Value);

impl std::fmt::Display for KeymapAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Value::String(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                let strings: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "{}", strings.join(", "))
            }
            _ => write!(f, "{}", self.0),
        }
    }
}
