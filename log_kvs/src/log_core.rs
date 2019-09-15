use std::collections::HashMap;
use std::path::Path;

use core::{Error, Result};

use crate::{Command, LogCommandPointer, LogFile};

/// An implementation of a key-value store using an append-only log store.
#[derive(Debug)]
pub struct LogKvs {
    pub(crate) index: HashMap<String, LogCommandPointer>,
    pub(crate) log: LogFile,
}

impl LogKvs {
    pub(crate) const DEFAULT_LOG_NAME: &'static str = "1";
    pub(crate) const DEFAULT_LOG_ID: usize = 1;

    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = Path::new(path.as_ref());
        let default_file = path.join(Self::DEFAULT_LOG_NAME);

        let kvs = LogKvs {
            index: HashMap::new(),
            log: LogFile::new(default_file),
        };

        Ok(kvs)
    }

    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = Path::new(path.as_ref());
        let default_file = path.join(Self::DEFAULT_LOG_NAME);

        let mut kvs = LogKvs {
            index: HashMap::new(),
            log: LogFile::new(default_file),
        };

        for record in kvs.log.iter()? {
            let (command, pointer) = record?;
            // println!("replaying {:?}, {:?}", command, pointer);
            kvs.replay(command, pointer)?;
        }
        Ok(kvs)
    }

    fn replay(
        &mut self,
        command: Command,
        pointer: LogCommandPointer,
    ) -> Result<()> {
        match command {
            Command::Set { key, .. } => {
                self.index.insert(key, pointer);
            }
            Command::Remove { key } => {
                self.index.remove(&key).ok_or_else(|| {
                    Error::corrupt_database(format!(
                        "attempted removal of nonexistent key '{}' from the \
                         index",
                        key
                    ))
                })?;
            }
        }
        Ok(())
    }

    pub(crate) fn get_key(
        &self,
        pointer: &LogCommandPointer,
    ) -> Result<String> {
        match self.log.get_command(pointer)? {
            Command::Set { value, .. } => Ok(value),
            Command::Remove { key } => Err(Error::corrupt_database(format!(
                "Command at {:?} should set key '{}', not remove it",
                pointer, key
            ))),
        }
    }
}
