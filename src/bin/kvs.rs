use clap::{App, AppSettings, Arg, SubCommand};
use strum_macros::{Display, EnumString};

// use kvs::KvStore;

fn main() {
    let mode = get_mode();
    println!("mode: {:?}", mode);

    eprintln!("unimplemented");
    unimplemented!();

    // let mut kvs = KvStore::new();

    // match mode {
    //     Mode::Get(key) => {
    //         kvs.get(key);
    //     }
    //     Mode::Set(key, value) => kvs.set(key, value),
    //     Mode::Remove(key) => kvs.remove(key),
    // }
}

fn get_mode() -> Mode {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("get")
                .about("retreive a value from the key-value store")
                .arg(
                    Arg::with_name("key")
                        .help("the item to retreive the value of")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("add a value to the key-value store")
                .arg(
                    Arg::with_name("key")
                        .help("the name to store the value under")
                        .required(true),
                )
                .arg(
                    Arg::with_name("value")
                        .help("the value to store")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("remove a value from the key-value store")
                .arg(
                    Arg::with_name("key")
                        .help("the item to delete")
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("get", Some(mode_matches)) => Mode::Get(String::from(
            mode_matches
                .value_of("key")
                .expect("subcommand get has no argument 'key'"),
        )),
        ("set", Some(mode_matches)) => Mode::Set(
            String::from(
                mode_matches
                    .value_of("key")
                    .expect("subcommand set has no argument 'key'"),
            ),
            String::from(
                mode_matches
                    .value_of("value")
                    .expect("subcommand set has no argument 'value'"),
            ),
        ),
        ("rm", Some(mode_matches)) => Mode::Remove(String::from(
            mode_matches
                .value_of("key")
                .expect("subcommand rm has no argument 'key'"),
        )),
        _ => unreachable!(), // one subcommand is required
    }
}

#[derive(Debug, Display, EnumString)]
enum Mode {
    #[strum(serialize = "get", serialize = "g")]
    Get(String),
    #[strum(serialize = "set", serialize = "s")]
    Set(String, String),
    #[strum(serialize = "remove", serialize = "rm", serialize = "r")]
    Remove(String),
}
