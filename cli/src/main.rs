use std::path::PathBuf;

use structopt::StructOpt;
#[macro_use]
extern crate strum_macros;
use strum_macros::Display;

use core::Result;
use hashmap_kvs::HashMapKvs;
use log_kvs::LogKvs;

mod command;
use command::{Command, Commandable};

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut store: Box<dyn Commandable> = match opt.store {
        Store::HashMap => Box::new(HashMapKvs::open(opt.location).unwrap()),
        Store::Log => Box::new(LogKvs::open(opt.location).unwrap()),
    };
    store.execute(opt.command)
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// Which type of backing store to use.
    #[structopt(short, long, default_value = "hashmap")]
    store: Store,
    /// The location to load and save the backing store.
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "../target/store"
    )]
    location: PathBuf,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, Display, EnumString, StructOpt)]
enum Store {
    /// Use a hashmap backed to the given file location.
    #[strum(serialize = "hashmap")]
    HashMap,
    /// Use an append-only log store backed in the given directory location.
    #[strum(serialize = "log")]
    Log,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::prelude::*;
    use predicates::ord::eq;
    use predicates::str::{contains, is_empty, PredicateStrExt};
    use std::process::Command;
    use tempfile::TempDir;

    use core::KvStore;

    // `kvs` with no args should exit with a non-zero code.
    #[test]
    fn cli_no_args() {
        Command::cargo_bin("cli").unwrap().assert().failure();
    }

    // `kvs -V` should print the version
    #[test]
    fn cli_version() {
        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-V"])
            .assert()
            .stdout(contains(env!("CARGO_PKG_VERSION")));
    }

    // `kvs get <KEY>` should print "Key not found" for a non-existent key and
    // exit with zero.
    #[test]
    fn cli_get_non_existent_key() {
        let temp_dir = TempDir::new().unwrap();
        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "get", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(eq("Key not found").trim());
    }

    // `kvs rm <KEY>` should print "Key not found" for an empty database and
    // exit with zero.
    #[test]
    fn cli_rm_non_existent_key() {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "rm", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(eq("Key not found").trim());
    }

    // `kvs set <KEY> <VALUE>` should print nothing and exit with zero.
    #[test]
    fn cli_set() {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "set", "key1", "value1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(is_empty());
    }

    #[test]
    fn cli_get_stored() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");

        let mut store = HashMapKvs::open(temp_dir.path().join("kvs_file"))?;
        store.set("key1".to_owned(), "value1".to_owned())?;
        store.set("key2".to_owned(), "value2".to_owned())?;
        drop(store);

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "get", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(eq("value1").trim());

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "get", "key2"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(eq("value2").trim());

        Ok(())
    }

    // `kvs rm <KEY>` should print nothing and exit with zero.
    #[test]
    fn cli_rm_stored() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");

        let mut store = HashMapKvs::open(temp_dir.path().join("kvs_file"))?;
        store.set("key1".to_owned(), "value1".to_owned())?;
        drop(store);

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "rm", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(is_empty());

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "get", "key1"])
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout(eq("Key not found").trim());

        Ok(())
    }

    #[test]
    fn cli_invalid_get() {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "get"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "get", "extra", "field"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }

    #[test]
    fn cli_invalid_set() {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "set"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "set", "missing_field"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "set", "extra", "extra", "field"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }

    #[test]
    fn cli_invalid_rm() {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "rm"])
            .current_dir(&temp_dir)
            .assert()
            .failure();

        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "rm", "extra", "field"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }

    #[test]
    fn cli_invalid_subcommand() {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        Command::cargo_bin("cli")
            .unwrap()
            .args(&["-l", "kvs_file", "unknown", "subcommand"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }
}
