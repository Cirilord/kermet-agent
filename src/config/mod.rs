mod error;
mod path;

pub(crate) use error::ConfigError;
pub(crate) use path::ensure_exists;
pub(crate) use path::read;

#[cfg(test)]
mod tests;
