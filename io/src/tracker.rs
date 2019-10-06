use std::io::{BufRead, Error, Read, Seek, SeekFrom, Write};

/// Represents a reader/writer/seeker that is aware of it's current position.
pub trait Trackable {
    /// Get the current position.
    fn current_pos(&self) -> u64;
}

/// A wrapper around any reader/writer/seeker that tracks its current position.
#[derive(Debug)]
pub struct Tracker<R> {
    item: R,
    pos: u64,
}

impl<R> Tracker<R> {
    /// Initializes the tracker. Assumes current location is pos 0.
    pub fn new(item: R) -> Self {
        Tracker { item, pos: 0 }
    }
}

impl<R: Read> Trackable for Tracker<R> {
    /// Get the current position.
    fn current_pos(&self) -> u64 {
        self.pos
    }
}

impl<R: Read> Read for Tracker<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self.item.read(buf) {
            Ok(amount_read) => {
                self.pos += amount_read as u64;
                Ok(amount_read)
            }
            Err(e) => Err(e),
        }
    }
}

impl<BR: BufRead> BufRead for Tracker<BR> {
    fn fill_buf(&mut self) -> Result<&[u8], Error> {
        self.item.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.pos += amount as u64;
        self.item.consume(amount);
    }
}

impl<W: Write> Write for Tracker<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let amount_written = self.item.write(buf)?;
        self.pos += amount_written as u64;
        Ok(amount_written)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.item.flush()
    }
}

impl<S: Seek> Seek for Tracker<S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error> {
        let new_pos = self.item.seek(pos)?;
        self.pos = new_pos;
        Ok(new_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{BufReader, Seek, SeekFrom, Write};

    use tempfile::tempfile;

    #[test]
    fn read() -> Result<(), Error> {
        // create test file and reset it to the beginning
        let mut file = tempfile()?;
        write!(file, "test\n")?;
        write!(file, "123\n")?;
        file.seek(SeekFrom::Start(0))?;

        // initialize Tracker
        let mut tracked_file = Tracker::new(file);
        assert_eq!(tracked_file.current_pos(), 0);

        // try reading from it
        let mut s = String::new();
        tracked_file.read_to_string(&mut s)?;
        assert_eq!(tracked_file.current_pos(), 9);

        Ok(())
    }

    #[test]
    fn read_buffered() -> Result<(), Error> {
        // create test file and reset it to the beginning
        let mut file = tempfile()?;
        write!(file, "test\n")?;
        write!(file, "123\n")?;
        file.seek(SeekFrom::Start(0))?;

        // initialize Tracker
        let file = BufReader::new(file);
        let mut tracked_file = Tracker::new(file);
        assert_eq!(tracked_file.current_pos(), 0);

        // try reading from it
        let mut s = String::new();
        tracked_file.read_line(&mut s)?;
        assert_eq!(tracked_file.current_pos(), 5);

        tracked_file.read_to_string(&mut s)?;
        assert_eq!(tracked_file.current_pos(), 9);

        Ok(())
    }

    #[test]
    fn write() -> Result<(), Error> {
        // create test file and reset it to the beginning
        let file = tempfile()?;
        let mut tracked_file = Tracker::new(file);
        assert_eq!(tracked_file.current_pos(), 0);

        write!(tracked_file, "test\n")?;
        assert_eq!(tracked_file.current_pos(), 5);

        write!(tracked_file, "123\n")?;
        assert_eq!(tracked_file.current_pos(), 9);

        Ok(())
    }

    #[test]
    fn seek() -> Result<(), Error> {
        // create test file and reset it to the beginning
        let mut file = tempfile()?;
        write!(file, "test\n")?;
        file.seek(SeekFrom::Start(0))?;

        let mut tracked_file = Tracker::new(file);
        assert_eq!(tracked_file.current_pos(), 0);

        tracked_file.seek(SeekFrom::End(0))?;
        assert_eq!(tracked_file.current_pos(), 5);

        tracked_file.seek(SeekFrom::Start(0))?;
        assert_eq!(tracked_file.current_pos(), 0);

        let mut s = String::new();
        tracked_file.read_to_string(&mut s)?;
        assert_eq!(s, "test\n");

        Ok(())
    }
}
