#[macro_export]
macro_rules! should_not_happen {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        {
            warn!("Expected to never happen, but got: {}", $msg);
        }
    };
}
