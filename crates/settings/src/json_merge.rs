#![allow(dead_code)]

/// Trait for recursively merging settings structures, ported from Zed's code base.
pub(crate) trait MergeFrom {
    /// Merge from a source of the same type.
    fn merge_from(&mut self, other: &Self);

    /// Merge from an optional source of the same type.
    fn merge_from_option(&mut self, other: Option<&Self>) {
        if let Some(other) = other {
            self.merge_from(other);
        }
    }
}

macro_rules! merge_from_overwrites {
    ($($type:ty),+ $(,)?) => {
        $(
            impl MergeFrom for $type {
                fn merge_from(&mut self, other: &Self) {
                    *self = other.clone();
                }
            }
        )+
    };
}

merge_from_overwrites!(
    u16,
    u32,
    u64,
    usize,
    i16,
    i32,
    i64,
    bool,
    f64,
    f32,
    String,
    std::path::PathBuf,
);

impl<T: Clone + MergeFrom> MergeFrom for Option<T> {
    fn merge_from(&mut self, other: &Self) {
        if let Some(other) = other {
            if let Some(this) = self {
                this.merge_from(other);
            } else {
                self.replace(other.clone());
            }
        }
    }
}

impl<T: Clone> MergeFrom for Vec<T> {
    fn merge_from(&mut self, other: &Self) {
        *self = other.clone();
    }
}

impl<T: MergeFrom> MergeFrom for Box<T> {
    fn merge_from(&mut self, other: &Self) {
        self.as_mut().merge_from(other.as_ref());
    }
}

impl<K, V> MergeFrom for std::collections::HashMap<K, V>
where
    K: Clone + std::hash::Hash + Eq,
    V: Clone + MergeFrom,
{
    fn merge_from(&mut self, other: &Self) {
        for (k, v) in other {
            if let Some(existing) = self.get_mut(k) {
                existing.merge_from(v);
            } else {
                self.insert(k.clone(), v.clone());
            }
        }
    }
}

impl<K, V> MergeFrom for std::collections::BTreeMap<K, V>
where
    K: Clone + Ord,
    V: Clone + MergeFrom,
{
    fn merge_from(&mut self, other: &Self) {
        for (k, v) in other {
            if let Some(existing) = self.get_mut(k) {
                existing.merge_from(v);
            } else {
                self.insert(k.clone(), v.clone());
            }
        }
    }
}

impl MergeFrom for serde_json::Value {
    fn merge_from(&mut self, other: &Self) {
        match (self, other) {
            (serde_json::Value::Object(this), serde_json::Value::Object(other)) => {
                for (k, v) in other {
                    if let Some(existing) = this.get_mut(k) {
                        existing.merge_from(v);
                    } else {
                        this.insert(k.clone(), v.clone());
                    }
                }
            }
            (this, other) => *this = other.clone(),
        }
    }
}
