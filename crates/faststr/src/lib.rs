use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub struct FastStr {
    inner: Arc<Inner>,
}
struct Inner {
    ptr: *const u8,
    counter: AtomicUsize,
    is_static: bool,
    len: usize,
}
unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

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
impl Clone for FastStr {
    fn clone(&self) -> Self {
        self.inner.counter.fetch_add(1, Ordering::SeqCst);
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<'a> FastStr {
    pub fn as_str(&'a self) -> &'a str {
        unsafe { std::mem::transmute::<_, &str>((self.inner.ptr, self.inner.len)) }
    }
}

impl Drop for FastStr {
    fn drop(&mut self) {
        let c = self.inner.counter.fetch_sub(1, Ordering::SeqCst) - 1;
        if !self.inner.is_static && c == 0 {
            unsafe {
                Vec::from_raw_parts(self.inner.ptr as *mut u8, self.inner.len, self.inner.len)
            };
        }
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

impl From<&'static str> for FastStr {
    fn from(value: &'static str) -> Self {
        let len = value.len();
        Self {
            inner: Arc::new(Inner {
                is_static: true,
                ptr: value.as_ptr(),
                counter: AtomicUsize::new(1),
                len,
            }),
        }
    }
}

impl From<String> for FastStr {
    fn from(value: String) -> Self {
        let len = value.len();
        let leaked = value.leak();
        Self {
            inner: Arc::new(Inner {
                is_static: false,
                ptr: leaked.as_ptr(),
                counter: AtomicUsize::new(1),
                len,
            }),
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
