use core::{Compactable, Result};

use crate::{Command, LogKvs};

impl Compactable for LogKvs {
    /// Compact the key-value store. Return an error if unsuccessful.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use core::{KvStore, Compactable, Persistent};
    /// # use log_kvs::LogKvs;
    ///
    /// # let temp_dir =
    /// #     TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = LogKvs::open(temp_dir.path()).unwrap();
    /// # store.set("key1".to_owned(), "value1".to_owned());
    /// # store.remove("key1".to_owned());
    /// store.compact();
    /// ```
    fn compact(&mut self) -> Result<()> {
        self.log.rewrite(|iter, mut writer| {
            for record in iter {
                let (command, pointer) = record?;
                match command {
                    Command::Set { key, value } => {
                        match self.index.get(&key) {
                            Some(current_pointer)
                                if pointer == *current_pointer =>
                            {
                                // this is a valid key and the current value
                                Command::Set { key, value }
                                    .append(&mut writer)?;
                            }
                            Some(_) => {
                                // this is a valid key, but not the current
                                // value
                            }
                            None => {
                                // invalid key
                            }
                        }
                    }
                    Command::Remove { .. } => {
                        // once removed, the key is no longer needed
                    }
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // generate_compactable_tests!(LogKvs);
}
