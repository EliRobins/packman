pub const SIGNATURE: &str = "PACKMAN: ";

#[macro_export]
macro_rules! log {
    ( $($x:expr),* ) => {
        {
            print!("{}", crate::logging::SIGNATURE);
            print!($($x)*);
        }
    }
}

#[macro_export]
macro_rules! logln {
    ( $($x:expr),* ) => {
        {
            print!("{}", crate::logging::SIGNATURE);
            println!($($x)*);
        }
    }
}

#[macro_export]
macro_rules! elog {
    ( $($x:expr),* ) => {
        {
            eprint!("{}", crate::logging::SIGNATURE);
            eprint!($($x)*);
        }
    }
}

#[macro_export]
macro_rules! elogln {
    ( $($x:expr),* ) => {
        {
            eprint!("{}", crate::logging::SIGNATURE);
            eprintln!($($x)*);
        }
    }
}
