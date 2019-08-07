use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use strum_macros::Display;

use super::DEFAULT_LOG_ID;
use crate::{Error, Result};

#[derive(Debug, Display, Serialize, Deserialize)]
pub(super) enum Command {
    /// Add a value to the key-value store.
    Set {
        /// The name to store the value under.
        key: String,
        /// The value to store.
        value: String,
    },
    /// Remove a value from the key-value store.
    Remove {
        /// The item to delete.
        key: String,
    },
}

impl Command {
    pub fn append<W: Write>(&self, writer: &mut W) -> Result<()> {
        bincode::serialize_into(writer, self).map_err(Error::bincode)
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Command> {
        bincode::deserialize_from(reader).map_err(Error::bincode)
    }
}

#[derive(Debug)]
pub(super) struct LogCommandPointer {
    file_id: usize,
    offset: u64,
}

impl LogCommandPointer {
    pub fn new(file_id: usize, offset: u64) -> LogCommandPointer {
        LogCommandPointer { file_id, offset }
    }
}

#[derive(Debug)]
pub(super) struct LogFile {
    path: PathBuf,
}

impl LogFile {
    pub fn new<P: AsRef<Path>>(path: P) -> LogFile {
        LogFile {
            path: PathBuf::from(path.as_ref()),
        }
    }

    pub fn iter(&self) -> Result<CommandLogIterator<File>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        CommandLogIterator::new(reader)
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
        Ok(LogCommandPointer::new(DEFAULT_LOG_ID, pos))
    }
}

pub(super) struct CommandLogIterator<R: Read + Seek> {
    reader: BufReader<R>,
    end_pos: u64,
}

impl<R: Read + Seek> CommandLogIterator<R> {
    fn new(mut reader: BufReader<R>) -> Result<CommandLogIterator<R>> {
        let end_pos = reader.stream_len()?;
        Ok(CommandLogIterator { reader, end_pos })
    }
}

impl<R: Read + Seek> Iterator for CommandLogIterator<R> {
    type Item = Result<(Command, LogCommandPointer)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.stream_position() {
            Ok(current_pos) if current_pos < self.end_pos => {
                Some(match Command::read(&mut self.reader) {
                    Ok(command) => Ok((
                        command,
                        LogCommandPointer::new(DEFAULT_LOG_ID, current_pos),
                    )),
                    Err(err) => Err(err),
                })
            }
            Ok(_) => None,
            Err(err) => Some(Err(Error::from(err))),
        }
    }
}
