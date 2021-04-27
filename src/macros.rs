#[macro_use]

#[macro_export]
macro_rules! unwrap_or_return {
    ( $e:expr, $err:expr  ) => {
        match $e {
            Some(x) => x,
            None => return Err($err),
        }
    }
}
