use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use core::{Persistent, Result};

/// An implementation of a key-value store using an in memory hashmap that
/// only saves the store on close.
#[derive(Debug)]
pub struct HashMapKvs {
    pub(crate) map: HashMap<String, String>,
    pub(crate) backing: PathBuf,
    pub(crate) mutated: bool,
}

impl HashMapKvs {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut kvs = HashMapKvs {
            map: HashMap::new(),
            backing: PathBuf::from(path.as_ref()),
            mutated: true,
        };

        kvs.save()?;
        Ok(kvs)
    }

    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let backing_file = File::open(&path)?;
        let reader = BufReader::new(backing_file);
        let map: HashMap<String, String> = serde_json::from_reader(reader)?;

        Ok(HashMapKvs {
            map,
            backing: PathBuf::from(path.as_ref()),
            mutated: false,
        })
    }
}
