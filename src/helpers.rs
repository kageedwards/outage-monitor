use std::result::Result as StdResult;
use std::error::Error as StdError;

// A macro to gracefully ignore println! statements if we aren't in debug mode.
#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

// A type alias to simplify error propagation from disparate sources (DRY).
pub type Result<T> = StdResult<T, Box<dyn StdError + Send + Sync>>;