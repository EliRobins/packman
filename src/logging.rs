pub const SIGNATURE: &str = "PACKMAN: ";
pub const LOG_COLOR: &str = "\x1b[33m";
pub const ERR_COLOR: &str = "\x1b[31m";

#[macro_export]
macro_rules! log {
    ( $($x:expr),* ) => {
        {
            print!("{}", crate::logging::LOG_COLOR);
            print!("{}", crate::logging::SIGNATURE);
            print!($($x)*);
            print!("\x1b[0m");
        }
    }
}

#[macro_export]
macro_rules! logln {
    ( $($x:expr),* ) => {
        {
            print!("{}", crate::logging::LOG_COLOR);
            print!("{}", crate::logging::SIGNATURE);
            print!($($x)*);
            println!("\x1b[0m");
        }
    }
}

#[macro_export]
macro_rules! elog {
    ( $($x:expr),* ) => {
        {
            eprint!("{}", crate::logging::ERR_COLOR);
            eprint!("{}", crate::logging::SIGNATURE);
            eprint!($($x)*);
            eprint!("\x1b[0m");
        }
    }
}

#[macro_export]
macro_rules! elogln {
    ( $($x:expr),* ) => {
        {
            eprint!("{}", crate::logging::ERR_COLOR);
            eprint!("{}", crate::logging::SIGNATURE);
            eprint!($($x)*);
            eprintln!("\x1b[0m");
        }
    }
}
