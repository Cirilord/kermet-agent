mod action;
mod command;
mod commands;
mod error;

pub(crate) use action::CliAction;
pub(crate) use command::parse;
pub(crate) use commands::{ConfigKeyInput, ConfigSetInput, LoginInput};
pub(crate) use error::CliError;

#[cfg(test)]
mod tests;
