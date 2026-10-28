use super::SandboxPlatform;

#[derive(Debug)]
pub enum SandboxError {
  Unavailable(SandboxPlatform),
  InvalidPolicy,
  Spawn(std::io::Error),
  MissingOutputStream,
  MissingErrorStream,
  Wait(std::io::Error),
  Kill(std::io::Error),
  Capture(std::io::Error),
  CaptureThread,
}
