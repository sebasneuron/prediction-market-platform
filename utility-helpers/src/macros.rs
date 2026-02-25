#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if $crate::SHOW_LOGS {
            tracing::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        if $crate::SHOW_LOGS {
            tracing::error!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if $crate::SHOW_LOGS {
            tracing::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        if $crate::SHOW_LOGS {
            tracing::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        if $crate::SHOW_LOGS {
            tracing::trace!($($arg)*);
        }
    };
}
