// use clap::{App, AppSettings, Arg, SubCommand};
use structopt::StructOpt;
use strum_macros::Display;

fn main() {
    let command = Command::from_args();
    println!("command: {:?}", command);

    eprintln!("unimplemented");
    unimplemented!();

    // let mut kvs = KvStore::new();

    // match command {
    //     Command::Get{ key } => {
    //         kvs.get(key);
    //     }
    //     Command::Set{ key, value } => kvs.set(key, value),
    //     Command::Remove{ key } => kvs.remove(key),
    // }
}

#[derive(Debug, Display, StructOpt)]
enum Command {
    #[structopt(name = "get")]
    /// Retrieve a value from the key-value store.
    Get {
        /// The item to retreive the value of.
        key: String,
    },
    #[structopt(name = "set")]
    /// Add a value to the key-value store.
    Set {
        /// The name to store the value under.
        key: String,
        /// The value to store.
        value: String,
    },
    #[structopt(name = "rm")]
    /// Remove a value from the key-value store.
    Remove {
        /// The item to delete.
        key: String,
    },
}
