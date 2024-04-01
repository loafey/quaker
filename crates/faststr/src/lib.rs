use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};

#[derive(Clone, Default)]
pub struct FastStr {
    inner: Arc<String>,
}

impl Deref for FastStr {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Serialize for FastStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for FastStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

impl PartialEq for FastStr {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Eq for FastStr {}
impl PartialOrd for FastStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FastStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}
impl Hash for FastStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Debug for FastStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl Display for FastStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'a> From<&'a str> for FastStr {
    fn from(value: &'a str) -> Self {
        Self {
            inner: Arc::new(value.to_string()),
        }
    }
}

impl From<String> for FastStr {
    fn from(value: String) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disp_format() {
        let og_string = (0..255u8).map(|a| a as char).collect::<String>();
        let fstring = FastStr::from(og_string.clone());
        assert_eq!(format!("{og_string:?}"), format!("{fstring:?}"))
    }

    #[test]
    fn dbg_format() {
        let og_string = (0..255u8).map(|a| a as char).collect::<String>();
        let fstring = FastStr::from(og_string.clone());
        assert_eq!(format!("{og_string:?}"), format!("{fstring:?}"))
    }

    #[test]
    fn string() {
        let test = FastStr::from("jacoboi".to_string());
        drop(test);
    }

    #[test]
    fn static_str() {
        let test = FastStr::from("jacoboi");
        drop(test);
    }
}
