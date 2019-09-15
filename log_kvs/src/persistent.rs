use std::path::Path;

use core::{Error, PathType, Persistent, Result};

use crate::LogKvs;

impl Persistent for LogKvs {
    const PATH_TYPE: PathType = PathType::Directory;

    fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = Path::new(path.as_ref());

        // create directory if need be
        if let Err(err) = std::fs::create_dir(path) {
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(Error::io(err));
            }
        }

        if path.join(Self::DEFAULT_LOG_NAME).is_file() {
            Self::load(path)
        } else {
            Self::new(path)
        }
    }

    fn save(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Drop for LogKvs {
    fn drop(&mut self) {
        self.save().expect("error saving LogKvs during drop");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    generate_persistent_tests!(LogKvs);
}
