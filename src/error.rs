use std::fmt;

#[derive(Debug)]
pub enum GameError {
  OutOfBoundsError { x: usize, y: usize },
  RepeatMoveError,
  InvalidMoveError,
}

impl std::error::Error for GameError {}

impl fmt::Display for GameError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      GameError::OutOfBoundsError { x, y } => {
        write!(f, "Out of Bounds Error for position ({}, {})", x, y)
      }
      GameError::RepeatMoveError => write!(f, "Repeated Move Error"),
      GameError::InvalidMoveError => write!(f, "Invalid Move Error"),
    }
  }
}
