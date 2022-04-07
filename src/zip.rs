use anyhow::{Context, Result};
use std::{
    fs,
    io::{BufWriter, Read, Write},
    path::Path,
};

/// `zip-rs` can't handle correctly, use command line temprary
pub(crate) fn unzip(apk: &Path, outdir: &Path) -> Result<()> {
    crate::cmd::unzip(apk, outdir).map(|_| ())
}

pub(crate) fn zip(src: &Path, dest: &Path) -> Result<()> {
    let mut writer = zip::ZipWriter::new(BufWriter::new(
        fs::File::create(dest).with_context(|| format!("{dest:?} create error"))?,
    ));

    let options = zip::write::FileOptions::default();

    let mut buffer = Vec::new();
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.context("dir entry error")?;
        let path = entry.path();
        let name = path.strip_prefix(src).context("path stirp error")?;
        if path.is_file() {
            #[allow(deprecated)]
            writer.start_file_from_path(name, options)?;

            buffer.clear();
            // there will be a memory issue if file is too big
            fs::File::open(path)?.read_to_end(&mut buffer)?;
            writer.write_all(buffer.as_ref())?;
        } else if !name.as_os_str().is_empty() {
            #[allow(deprecated)]
            writer.start_file_from_path(name, options)?;
        }
    }
    writer.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_zip() {
        let src = "/Users/bytedance/Downloads/app-china-debug";
        let dest = "/tmp/a.apk";
        super::zip(src.as_ref(), dest.as_ref()).unwrap()
    }
}
