use std::fs::File;
use std::io::BufWriter;

use core::{ExplicitlyPersistent, Result};
use io::safe_overwrite;

use crate::HashMapKvs;

impl ExplicitlyPersistent for HashMapKvs {
    fn save(&mut self) -> Result<()> {
        safe_overwrite(self.backing.clone(), |writer: BufWriter<File>| {
            serde_json::to_writer(writer, &self.map)?;
            self.mutated = false;
            Ok(())
        })
    }
}
