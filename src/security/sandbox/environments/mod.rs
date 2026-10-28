mod current;
#[cfg(target_os = "macos")]
mod macos;

pub use current::Sandbox;
#[cfg(target_os = "macos")]
pub(super) use macos::MacOsSandbox;
