use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ReadRequest {
  path: PathBuf,
  line_range: Option<LineRange>,
}

impl ReadRequest {
  pub fn new(path: PathBuf) -> Self {
    return Self {
      path,
      line_range: None,
    };
  }

  pub fn with_line_range(
    path: PathBuf,
    start: usize,
    end: usize,
  ) -> Result<Self, ReadRequestError> {
    let line_range = match LineRange::new(start, end) {
      Ok(line_range) => line_range,
      Err(error) => return Err(error),
    };

    return Ok(Self {
      path,
      line_range: Some(line_range),
    });
  }

  pub fn line_range(&self) -> Option<(usize, usize)> {
    let line_range = self.line_range.map(|range| return (range.start, range.end));

    return line_range;
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }
}

#[derive(Clone, Copy, Debug)]
struct LineRange {
  start: usize,
  end: usize,
}

impl LineRange {
  fn new(start: usize, end: usize) -> Result<Self, ReadRequestError> {
    if start == 0 || end < start {
      return Err(ReadRequestError::InvalidLineRange { start, end });
    }

    return Ok(Self { start, end });
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ReadRequestError {
  InvalidLineRange { start: usize, end: usize },
}
