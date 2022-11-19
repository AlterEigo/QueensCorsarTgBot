/// Универсальное возвращаемое значение с возможностью типизирования параметра
pub type UResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use qcproto::prelude::*;
pub use crate::core::*;
pub use crate::logger::*;
pub use crate::utility::*;
pub use slog::{crit, debug, error, info, o, warn};
