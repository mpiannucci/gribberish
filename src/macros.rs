
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
    ($e:expr, $sig_bit:expr, $dest:ident) => {
        
        if ($e & (1 << ($sig_bit - 1))) > 0 {
            -(($e & !(1 << ($sig_bit - 1))) as $dest)
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
        let neg_one: u8 = 0b0000000010000001;
        assert_eq!(as_signed!(neg_one, 8, i16), -1);

        let four: u8 = 0b00000100;
        assert_eq!(as_signed!(four, 8, i8), 4);
    }
}