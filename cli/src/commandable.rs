use core::{Compactable, KvStore, Result};
use hashmap_kvs::HashMapKvs;
use log_kvs::LogKvs;

use crate::args::Command;

pub(crate) trait Commandable: KvStore {
    fn execute_get(&self, key: String) -> Result<()> {
        let value = self.get(key)?;
        match value {
            Some(value) => println!("{}", value),
            None => println!("Key not found"),
        };
        Ok(())
    }

    fn execute_set(&mut self, key: String, value: String) -> Result<()> {
        self.set(key, value)
    }

    fn execute_rm(&mut self, key: String) -> Result<()> {
        if self.remove(key.clone())?.is_none() {
            println!("Key not found");
        }
        Ok(())
    }

    fn execute_compact(&mut self) -> Result<()> {
        println!("Compaction not supported on this type of store.");
        Ok(())
    }

    fn execute(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Get { key } => self.execute_get(key),
            Command::Set { key, value } => self.execute_set(key, value),
            Command::Remove { key } => self.execute_rm(key),
            Command::Compact => self.execute_compact(),
        }
    }
}

impl Commandable for HashMapKvs {}

impl Commandable for LogKvs {
    fn execute_compact(&mut self) -> Result<()> {
        self.compact()
    }
}
