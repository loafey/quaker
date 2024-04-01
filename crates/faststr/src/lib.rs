use std::{
    fmt::{Debug, Display},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub struct FastStr {
    inner: *const u8,
    counter: Arc<AtomicUsize>,
    is_static: bool,
    len: usize,
}
unsafe impl Send for FastStr {}
unsafe impl Sync for FastStr {}

impl Clone for FastStr {
    fn clone(&self) -> Self {
        self.counter.fetch_add(1, Ordering::Acquire);
        Self {
            inner: self.inner,
            counter: self.counter.clone(),
            is_static: self.is_static,
            len: self.len,
        }
    }
}

impl From<&'static str> for FastStr {
    fn from(value: &'static str) -> Self {
        let len = value.len();
        Self {
            is_static: true,
            inner: value.as_ptr(),
            counter: Arc::new(AtomicUsize::new(1)),
            len,
        }
    }
}

impl From<String> for FastStr {
    fn from(value: String) -> Self {
        let len = value.len();
        let leaked = value.leak();
        Self {
            is_static: false,
            inner: leaked.as_ptr(),
            counter: Arc::new(AtomicUsize::new(1)),
            len,
        }
    }
}
impl Drop for FastStr {
    fn drop(&mut self) {
        let c = self.counter.fetch_sub(1, Ordering::Acquire) - 1;
        if !self.is_static && c == 0 {
            unsafe { Vec::from_raw_parts(self.inner as *mut u8, self.len, self.len) };
        }
    }
}
impl Debug for FastStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe {
            std::mem::transmute::<_, &str>((self.inner, self.len))
        })
    }
}
impl Display for FastStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe {
            std::mem::transmute::<_, &str>((self.inner, self.len))
        })
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
