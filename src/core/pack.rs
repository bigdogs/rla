use anyhow::{format_err, Context, Result};
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};
use tracing::instrument;

use crate::{
    deps::SMALI,
    dir::{binarydir, temppath},
};

#[instrument(skip_all, level = "debug", fields(dex=dex.file_name().unwrap().to_str().unwrap()))]
async fn smali(smali_dir: PathBuf, dex: PathBuf, smali_jar: PathBuf) -> Result<()> {
    let tmp = temppath(dex.file_name().context("path invalid")?);
    crate::cmd::smali(&smali_dir, &tmp, &smali_jar)?;

    fs::copy(tmp, dex).with_context(|| "copy error".to_string())?;
    Ok(())
}

fn entries(root: &Path) -> io::Result<Vec<PathBuf>> {
    fs::read_dir(root)?
        .map(|r| r.map(|e| e.path()))
        .collect::<io::Result<Vec<_>>>()
}

/// (smali_dir, dex)
fn smali_mapping_dex(smali_dir: PathBuf, apk_unpacked: &Path) -> Result<(PathBuf, PathBuf)> {
    let dex_name = smali_dir
        .file_name()
        .with_context(|| format!("file name invalid {smali_dir:?}"))?;
    let dex = apk_unpacked.join(dex_name);
    if !dex.exists() {
        return Err(format_err!("{dex:?} not exists"));
    }
    Ok((smali_dir, dex))
}

#[instrument(skip_all, level = "debug")]
async fn task_sync_smalis(root: PathBuf) -> Result<()> {
    let smalis_dir = root.join(super::SMALIS);
    let unpacked_dir = root.join(super::UNPACKED);
    let smali_jar = SMALI.release_binary(binarydir())?;

    let handles = entries(&smalis_dir)
        .with_context(|| format!("read dir {root:?} error"))?
        .into_iter()
        .map(|smali_dir| smali_mapping_dex(smali_dir, &unpacked_dir))
        .collect::<Result<Vec<(PathBuf, PathBuf)>>>()?
        .into_iter()
        .map(|(smali_dir, dex)| tokio::spawn(smali(smali_dir, dex, smali_jar.clone())))
        .collect::<Vec<_>>();
    for h in handles {
        let _r = h.await??;
    }
    Ok(())
}

fn next_apk_name(dir: &Path) -> String {
    let idx = walkdir::WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(OsStr::to_str)
                .and_then(|s| s.parse::<usize>().ok())
        })
        .max()
        .unwrap_or(0);
    format!("{}.apk", idx + 1)
}

#[instrument(skip_all, level = "debug")]
async fn task_zip(root: PathBuf) -> Result<PathBuf> {
    let unpacked_dir = root.join(super::UNPACKED);
    let out_dir = root.join("output");
    if !out_dir.exists() {
        fs::create_dir(&out_dir).with_context(|| format!("{out_dir:?} create error"))?;
    }
    let apk = out_dir.join(next_apk_name(&out_dir));
    crate::zip::zip(&unpacked_dir, &apk).context("zip error")?;
    Ok(apk)
}

#[instrument(skip_all, level = "debug")]
async fn task_sign(apk: PathBuf) -> Result<()> {
    crate::cmd::debugsign(&apk)
}

pub(crate) async fn run(root: PathBuf) -> Result<()> {
    task_sync_smalis(root.clone()).await?;
    let apk = task_zip(root.clone()).await?;
    task_sign(apk).await?;
    Ok(())
}
