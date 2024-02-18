#[macro_export]
macro_rules! error_return {
    ($context:literal) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                error!("{e}");
                return;
            }
        }
    }};
    ($context:expr) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                error!("{e}");
                return;
            }
        }
    }};
}
