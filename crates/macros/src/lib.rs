#[macro_export]
macro_rules! error_return {
    ($context:literal) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                error!("{e}");
                return Default::default();
            }
        }
    }};
    ($context:expr) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                bevy::log::error!("{e}");
                return Default::default();
            }
        }
    }};
}

#[macro_export]
macro_rules! error_continue {
    ($context:literal) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                error!("{e}");
                continue;
            }
        }
    }};
    ($context:expr) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                bevy::log::error!("{e}");
                continue;
            }
        }
    }};
}

#[macro_export]
macro_rules! option_return {
    ($context:literal) => {{
        match $context {
            Some(map) => map,
            None => {
                return Default::default();
            }
        }
    }};
    ($context:expr) => {{
        match $context {
            Some(map) => map,
            None => {
                return Default::default();
            }
        }
    }};
}

#[macro_export]
macro_rules! npdbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        eprintln!("[{}:{}:{}]", file!(), line!(), column!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                eprintln!("[{}:{}:{}] {} = {:?}",
                    file!(), line!(), column!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(dbg!($val)),+,)
    };
}
