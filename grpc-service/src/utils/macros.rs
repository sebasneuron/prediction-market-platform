#[macro_export]
macro_rules! validate_strings {
    ( $($input:expr),* ) => {
        $(
            if $input.is_empty() {
                let error_message = format!("{} cannot be empty", stringify!($input));
                return Err(tonic::Status::invalid_argument(error_message));
            }
        )*
    };
}

#[macro_export]
macro_rules! validate_numbers {
    ( $($input:expr),* ) => {
        $(
            if $input == 0 {
                let error_message = format!("{} cannot be 0", stringify!($input));
                return Err(tonic::Status::invalid_argument(error_message));
            }
        )*
    };
}
