use crate::domain::Size;

#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Window size {requested:?} is below minimum {min:?}")]
    SizeTooSmall { requested: Size, min: Size },
}
