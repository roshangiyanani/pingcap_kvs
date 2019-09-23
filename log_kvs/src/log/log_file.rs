use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek};
use std::path::{Path, PathBuf};

use core::{Error, Result};
use io::save_overwrite_with_reader;

use super::{Command, LogCommandPointer};
use crate::LogKvs;

#[derive(Debug)]
pub(crate) struct LogFile {
    path: PathBuf,
}

impl LogFile {
    pub fn new<P: AsRef<Path>>(path: P) -> LogFile {
        LogFile {
            path: PathBuf::from(path.as_ref()),
        }
    }

    pub fn iter(&self) -> Result<LogFileIterator<File>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        LogFileIterator::new(reader)
    }

    pub fn get_command(&self, pointer: &LogCommandPointer) -> Result<Command> {
        let mut file = File::open(&self.path)?;
        file.seek(std::io::SeekFrom::Start(pointer.offset))?;
        let mut reader = BufReader::new(file);
        Command::read(&mut reader)
    }

    pub fn append(&self, command: Command) -> Result<LogCommandPointer> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(file);
        let pos = writer.seek(std::io::SeekFrom::End(0))?;
        command.append(&mut writer)?;
        Ok(LogCommandPointer::new(LogKvs::DEFAULT_LOG_ID, pos))
    }

    pub fn rewrite<F>(&self, write_func: F) -> Result<()>
    where
        F: FnOnce(LogFileIterator<File>, BufWriter<File>) -> Result<()>,
    {
        save_overwrite_with_reader(&self.path, |reader, writer| {
            write_func(LogFileIterator::new(reader)?, writer)
        })
    }
}

pub(crate) struct LogFileIterator<R: Read + Seek> {
    reader: BufReader<R>,
    end_pos: u64,
}

impl<R: Read + Seek> LogFileIterator<R> {
    pub fn new(mut reader: BufReader<R>) -> Result<LogFileIterator<R>> {
        let end_pos = reader.stream_len()?;
        Ok(LogFileIterator { reader, end_pos })
    }
}

impl<R: Read + Seek> Iterator for LogFileIterator<R> {
    type Item = Result<(Command, LogCommandPointer)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.stream_position() {
            Ok(current_pos) if current_pos < self.end_pos => {
                Some(match Command::read(&mut self.reader) {
                    Ok(command) => Ok((
                        command,
                        LogCommandPointer::new(
                            LogKvs::DEFAULT_LOG_ID,
                            current_pos,
                        ),
                    )),
                    Err(err) => Err(err),
                })
            }
            Ok(_) => None,
            Err(err) => Some(Err(Error::from(err))),
        }
    }
}
