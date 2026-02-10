#[derive(Debug, serde::Serialize)]
#[cfg_attr(feature = "gui", derive(specta::Type))]
pub struct CommandError(pub String);

impl<E> From<E> for CommandError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(format!("{:#}", err.into()))
    }
}

pub type Result<T> = std::result::Result<T, CommandError>;
