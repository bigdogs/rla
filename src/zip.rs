use anyhow::{Context, Result};
use std::{
    fs, io,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

pub(crate) fn unzip(apk: &Path, outdir: &Path) -> Result<()> {
    let reader = BufReader::new(fs::File::open(&apk)?);
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => outdir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut writer = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut writer)?;
        }
    }
    Ok(())
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
