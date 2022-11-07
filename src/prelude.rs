/// Универсальное возвращаемое значение с возможностью типизирования параметра
pub type UResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use crate::logger::*;
pub use crate::core::*;
pub use crate::utility::*;
pub use slog::{o, info, error, warn, crit, debug};
