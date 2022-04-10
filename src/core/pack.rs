use anyhow::{format_err, Context, Result};
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};
use tempfile::TempPath;
use tracing::instrument;

use crate::{
    deps::SMALI,
    dir::{binarydir, temppath},
};

use super::RlaConfig;

use tracing::debug;

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
fn smali_mapping_dex(smali_dir: PathBuf, dex_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let dex_name = smali_dir
        .file_name()
        .with_context(|| format!("file name invalid {smali_dir:?}"))?;
    let dex = dex_dir.join(dex_name);
    Ok((smali_dir, dex))
}

#[instrument(skip_all, level = "debug")]
async fn smalis_to_dex(root: PathBuf) -> Result<TempPath> {
    let dex_dir = temppath("tmpdex");
    fs::create_dir_all(&dex_dir).context("{dex_dir:? create error}")?;

    let smali_jar = SMALI.release_binary(binarydir())?;

    let smalis_dir = root.join(super::SMALIS);
    let handles = entries(&smalis_dir)
        .with_context(|| format!("read dir {root:?} error"))?
        .into_iter()
        .map(|smali_dir| smali_mapping_dex(smali_dir, &dex_dir))
        .collect::<Result<Vec<(PathBuf, PathBuf)>>>()?
        .into_iter()
        .map(|(smali_dir, dex)| tokio::spawn(smali(smali_dir, dex, smali_jar.clone())))
        .collect::<Vec<_>>();

    debug!("there is {} dex files", handles.len());
    for h in handles {
        let _r = h.await??;
    }
    Ok(dex_dir)
}

fn next_output_apk(root: &Path) -> Result<PathBuf> {
    let out_dir = root.join("output");
    if !out_dir.exists() {
        fs::create_dir(&out_dir).with_context(|| format!("{out_dir:?} create error"))?;
    }
    let idx = walkdir::WalkDir::new(&out_dir)
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
    Ok(out_dir.join(format!("{}.apk", idx + 1)))
}

fn get_dex_names(dex_dir: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(dex_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "dex").unwrap_or(false))
        .filter_map(|e| e.path().file_name().map(PathBuf::from))
        .collect::<Vec<_>>()
}

#[instrument(skip_all, level = "debug")]
async fn task_sign(apk: PathBuf) -> Result<()> {
    crate::cmd::debugsign(&apk)
}

async fn task_sync_smali_to_apk(root: &Path, dex_dir: &Path) -> Result<PathBuf> {
    let next_apk = next_output_apk(root)?;
    let bak_apk = root.join(super::BAK_APK);
    fs::copy(&bak_apk, &next_apk)
        .with_context(|| format!("copy {bak_apk:?} to {next_apk:?} error"))?;

    let dex_names = get_dex_names(dex_dir);
    crate::cmd::zip_update_files(&next_apk, dex_dir, &dex_names)?;
    Ok(next_apk)
}

async fn task_sync_smali_full(root: &Path, dex_dir: &Path) -> Result<PathBuf> {
    let unpacked = root.join(super::UNPACKED);
    for dex in get_dex_names(dex_dir) {
        let origin_dex = unpacked.join(&dex);
        if origin_dex.exists() {
            return Err(format_err!("{origin_dex:?} not exists"));
        }
        fs::copy(dex_dir.join(&dex), origin_dex).context("copy error")?;
    }

    let next_apk = next_output_apk(root)?;
    crate::zip::zip(&unpacked, &next_apk).context("zip error")?;
    Ok(next_apk)
}

pub(crate) async fn run(root: PathBuf) -> Result<()> {
    let config =
        fs::read_to_string(root.join(super::RLA_CONFIG)).context("rla config read error")?;
    let config: RlaConfig = serde_json::from_str(&config).context("config parse error")?;
    debug!("config is {config:?}");

    let dex_dir = smalis_to_dex(root.clone()).await?;
    let apk = if config.smali_only {
        task_sync_smali_to_apk(&root, dex_dir.as_ref()).await?
    } else {
        task_sync_smali_full(&root, dex_dir.as_ref()).await?
    };
    task_sign(apk).await?;

    Ok(())
}
