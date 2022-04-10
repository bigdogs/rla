use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{format_err, Context, Result};
use tokio::spawn;
use tracing::{error, instrument};

use crate::{
    deps::{BAKSMALI, FRIDA_INDEX_JS, FRIDA_PACKAGE, GIT_IGNORE},
    dir::{binarydir, temppath},
};

use super::{RlaConfig, RLA_CONFIG};

#[instrument(skip_all, level = "debug")]
async fn task_prepare_files(outdir: PathBuf, apk: PathBuf, config: RlaConfig) -> Result<()> {
    let bak = outdir.join(super::BAK_APK);
    fs::copy(&apk, &bak)?;
    GIT_IGNORE.release_binary(&outdir)?;
    // currently , we don't have any config, just use a file to identifier the project root dir
    fs::write(
        outdir.join(RLA_CONFIG),
        serde_json::to_string_pretty(&config)?,
    )?;

    // prepare mini firda
    let mini_frida = outdir.join(super::MINI_FRIDA);
    fs::create_dir(&mini_frida).with_context(|| format!("{mini_frida:?} create failed"))?;
    FRIDA_INDEX_JS.release_binary(&mini_frida)?;
    FRIDA_PACKAGE.release_binary(&mini_frida)?;
    Ok(())
}

#[instrument(skip_all, level = "debug", fields(dex=dex.file_name().unwrap().to_str().unwrap()))]
async fn task_baksmali(dex: PathBuf, smalis_dir: PathBuf, baksmali_jar: PathBuf) -> Result<String> {
    let dexname = dex.file_name().context("dex file no name")?;
    let outdir = smalis_dir.join(dexname);
    crate::cmd::baksmali(&dex, &outdir, &baksmali_jar)
}

async fn task_dex_to_smali(dex_dir: &Path, outdir: &Path) -> Result<()> {
    let dexes = walkdir::WalkDir::new(dex_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();
    if dexes.is_empty() {
        return Err(format_err!("no dex found"))?;
    }

    let smalis = outdir.join(super::SMALIS);
    fs::create_dir(&smalis).with_context(|| format!("{smalis:?} create error"))?;
    let baksmali_jar = BAKSMALI.release_binary(binarydir())?;
    let handles = dexes
        .into_iter()
        .filter(|p| p.is_file() && p.extension().eq(&Some(OsStr::new("dex"))))
        .map(|dex| tokio::spawn(task_baksmali(dex, smalis.clone(), baksmali_jar.clone())))
        .collect::<Vec<_>>();
    for h in handles {
        h.await??;
    }
    Ok(())
}

#[instrument(skip_all, level = "debug")]
async fn task_extract_all(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    let unpacked = outdir.join(super::UNPACKED);
    crate::cmd::unzip(&apk, &unpacked)?;

    task_dex_to_smali(unpacked.as_ref(), &outdir).await?;

    Ok(())
}

#[instrument(skip_all, level = "debug")]
async fn task_extract_smali(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    let temp_dexs = temppath("tmpdex");
    crate::zip::unzip(
        &apk,
        &temp_dexs,
        Some(|name: &Path| {
            name.parent().map(|s| s.as_os_str() == "").unwrap_or(true)
                && name.extension().map(|s| s == "dex").unwrap_or(false)
        }),
    )
    .context("unzip error")?;

    task_dex_to_smali(temp_dexs.as_ref(), &outdir).await?;

    Ok(())
}

#[instrument(skip_all, level = "debug")]
async fn task_git_init(outdir: PathBuf) -> Result<()> {
    if let Err(e) = crate::cmd::git_init(&outdir) {
        // Notice user that git is not available, but not fail the procedure
        error!("{e:?}");
    }
    Ok(())
}

#[instrument(skip_all, level = "debug")]
async fn task_jadx_reverse(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    if let Err(e) = crate::cmd::jadx_extract_src(&apk, &outdir.join(super::JADX_SRC)) {
        error!("{e:?}");
    }
    Ok(())
}

#[instrument(skip_all, level = "debug")]
async fn task_git_commit(outdir: PathBuf, msg: String) {
    if let Err(e) = crate::cmd::git_add(&outdir).and_then(|_| crate::cmd::git_commit(&outdir, &msg))
    {
        error!("{e:?}");
    }
}

pub(crate) async fn run(outdir: PathBuf, apk: PathBuf, config: RlaConfig) -> Result<()> {
    // >> base.apk
    // >> unzip >> smali
    // >> git init
    // >> jadx
    // >> .gitginore, .rla.config.json
    // ====
    // >> git commit

    // parallel tasks begin
    let mut handles = vec![spawn(task_prepare_files(
        outdir.clone(),
        apk.clone(),
        config.clone(),
    ))];

    if config.smali_only {
        handles.push(spawn(task_extract_smali(outdir.clone(), apk.clone())));
    } else {
        handles.push(spawn(task_extract_all(outdir.clone(), apk.clone())));
    }

    if config.git_enable {
        handles.push(spawn(task_git_init(outdir.clone())));
    }
    if config.jadx_enable {
        handles.push(spawn(task_jadx_reverse(outdir.clone(), apk.clone())));
    }
    for h in handles {
        h.await??;
    }

    if config.git_enable {
        task_git_commit(outdir, "Frist init project".to_string()).await;
    }
    Ok(())
}
