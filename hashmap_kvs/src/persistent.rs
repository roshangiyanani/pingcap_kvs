use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use core::{PathType, Persistent, Result};
use io::safe_overwrite;

use crate::HashMapKvs;

impl Persistent for HashMapKvs {
    const PATH_TYPE: PathType = PathType::File;

    fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().is_file() {
            HashMapKvs::load(path)
        } else {
            HashMapKvs::new(path)
        }
    }

    fn save(&mut self) -> Result<()> {
        safe_overwrite(self.backing.clone(), |writer: BufWriter<File>| {
            serde_json::to_writer(writer, &self.map)?;
            self.mutated = false;
            Ok(())
        })
    }
}

impl Drop for HashMapKvs {
    fn drop(&mut self) {
        if self.mutated {
            self.save().expect("error saving HashMapKvs during drop");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    generate_persistent_tests!(HashMapKvs);
}
