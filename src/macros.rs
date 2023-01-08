
#[macro_export]
macro_rules! unwrap_or_return {
    ( $e:expr, $err:expr  ) => {
        match $e {
            Some(x) => x,
            None => return Err($err),
        }
    }
}

#[macro_export]
macro_rules! as_signed{
    ($e:expr, $dest:ident) => {
        if $e.leading_ones() > 0 {
            -(($e << 1 >> 1) as $dest)
        } else {
            $e as $dest
        }
    };
}