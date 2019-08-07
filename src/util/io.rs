use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::Result;

/// Used to write a file without overwriting the file that already existed. Does
/// so by writing to a temporary file, then renaming the actual file to
/// something different, then renaming the new one.
pub(crate) fn safe_overwrite<P: AsRef<Path>, F>(
    path: P,
    write_func: F,
) -> Result<()>
where
    F: FnOnce(BufWriter<File>) -> Result<()>,
{
    let target = Path::new(path.as_ref());
    let tmp = target.with_extension("tmp");
    let backup = target.with_extension("backup");

    // if this succeeds, file never exists and we can write directly to it,
    // removing the need to rename.
    let (writer, direct) = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
    {
        Err(ref err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&tmp)
                .map(|writer| (writer, false))
        }
        other => other.map(|writer| (writer, true)),
    }?;
    let writer = BufWriter::new(writer);
    write_func(writer)?;

    if !direct {
        std::fs::rename(&target, &backup)?;
        std::fs::rename(&tmp, &target)?;
        std::fs::remove_file(&backup)?;
    }

    Ok(())
}

/// Similar to save_overwrite, but gives the function a reader to the old file.
pub(crate) fn save_overwrite_with_reader<P: AsRef<Path>, F>(
    path: P,
    write_func: F,
) -> Result<()>
where
    F: FnOnce(BufReader<File>, BufWriter<File>) -> Result<()>,
{
    let target = Path::new(path.as_ref());
    let tmp = target.with_extension("tmp");
    let backup = target.with_extension("backup");

    let reader = File::open(&target)?;
    let reader = BufReader::new(reader);
    let writer = File::create(&tmp)?;
    let writer = BufWriter::new(writer);
    write_func(reader, writer)?;

    std::fs::rename(&target, &backup)?;
    std::fs::rename(&tmp, &target)?;
    std::fs::remove_file(&backup)?;

    Ok(())
}
