use anyhow::{format_err, Context, Result};
use std::{fs, path::Path};

use crate::dir::temppath;

pub fn smali_to_java(path: &str) -> Result<()> {
    let path = Path::new(path);
    if !(path.is_file() && path.extension().map(|s| s == "smali").unwrap_or(false)) {
        return Err(format_err!("{path:?} is not a smali file"));
    }

    let dest = path.with_extension("java");
    let dest_name = dest.file_name().context("invalid file name")?;

    let tmpdir = temppath("tmp.java");
    let tmpdir: &Path = tmpdir.as_ref();
    crate::cmd::jadx_compile_smali(path, tmpdir)?;
    let java_file = walkdir::WalkDir::new(tmpdir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().file_name().eq(&Some(dest_name)))
        .map(|e| e.path().to_path_buf())
        .next()
        .with_context(|| format!(".java not found at {tmpdir:?}"))?;
    fs::rename(&java_file, &dest)
        .with_context(|| format!("fs rename {dest:?} to {java_file:?} error"))?;

    Ok(())
}
