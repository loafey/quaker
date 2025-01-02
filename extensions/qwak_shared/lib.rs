#[qwak_macro::plugin]
pub trait QwakPlugin {
    fn plugin_name() -> String {
        todo!("plugin_name not implemented");
    }
}

pub enum QwakError {
    Poison(String),
    Extism(extism::Error),
}
impl From<extism::Error> for QwakError {
    fn from(value: extism::Error) -> Self {
        Self::Extism(value)
    }
}
impl<T> From<std::sync::PoisonError<T>> for QwakError {
    fn from(value: std::sync::PoisonError<T>) -> Self {
        Self::Poison(format!("{value}"))
    }
}
