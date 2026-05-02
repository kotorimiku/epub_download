#[derive(Debug, serde::Serialize)]
#[cfg_attr(feature = "gui", derive(specta::Type))]
pub struct CommandError(pub String);

impl<E> From<E> for CommandError
where
    E: Into<color_eyre::eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(format!("{:#}", err.into()))
    }
}

pub type Result<T, E = color_eyre::eyre::Error> = color_eyre::eyre::Result<T, E>;

#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        ::color_eyre::eyre::bail!($($arg)*)
    };
}

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {
        ::color_eyre::eyre::eyre!($($arg)*)
    };
}
