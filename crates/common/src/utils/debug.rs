#[macro_export]
macro_rules! should_not_happen {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            warn!("[Should not happen] {}", format!($($arg)*));
        }
    };
}
