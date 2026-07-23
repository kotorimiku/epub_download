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
