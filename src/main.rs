use packman::{
    Config,
    Error,
    run
};

use std::{
    process,
    env,
};

fn main() {
    let args = env::args();
    let config = Config::build(args).unwrap_or_else(|err| {
        Error::print(&err);
        process::exit(1);
    });

    if let Err(err) = run(config) {
        Error::print(&err);
        process::exit(1);
    }
}
