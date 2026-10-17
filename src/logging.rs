#[macro_export]
macro_rules! log {
    ( $($x:expr),* ) => {
        {
            print!("PACKMAN: ");
            print!($($x)*);
        }
    }
}

#[macro_export]
macro_rules! logln {
    ( $($x:expr),* ) => {
        {
            print!("PACKMAN: ");
            println!($($x)*);
        }
    }
}
