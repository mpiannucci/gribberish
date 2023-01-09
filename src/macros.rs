
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

#[cfg(test)]
mod tests {
    // use super::*;


    #[test]
    fn test_convert_signed() {
        let neg_one: u8 = 0b10000001;
        assert_eq!(as_signed!(neg_one, i8), -1);

        let four: u8 = 0b00000100;
        assert_eq!(as_signed!(four, i8), 4);
    }
}