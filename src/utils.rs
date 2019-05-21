macro_rules! impl_error {
    ($error:ident {$($arg_name:ident: $arg_type:ty),*}) => {
        pub struct $error {
            $(
                pub $arg_name: $arg_type
            ),*
        }

        impl $error {
            pub fn new($($arg_name: $arg_type),*) -> Box<std::error::Error> {
                Box::new( Self {
                    $($arg_name),*
                })
            }
        }

        impl std::error::Error for $error {}

        impl std::fmt::Display for $error {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                write!(f,
                    concat!("{} with [", concat!($(stringify!($arg_name), ": {}; "),*), "]"),
                    stringify!($error),
                    $(self.$arg_name),*
                )?;

                Ok(())
            }
        }

        impl std::fmt::Debug for $error {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                write!(f,
                    concat!("{} with [", concat!($(stringify!($arg_name), ": {}; "),*), "]"),
                    stringify!($error),
                    $(self.$arg_name),*
                )?;

                Ok(())
            }
        }
    }
}
