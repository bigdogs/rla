use std::{ffi::OsStr, fs, path::PathBuf};

use anyhow::{Context, Result};
use tokio::spawn;
use tracing::{error, instrument};

use crate::{deps::BAKSMALI, dir::binarydir};

use super::RLA_CONFIG;

#[instrument(skip_all)]
async fn task_prepare_files(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    let bak = outdir.join("bak.apk");
    fs::copy(&apk, &bak)?;

    fs::write(
        outdir.join(".gitignore"),
        r#"output
**.DS_Store
**.gradle/
**.idea/
**gradle/
**gradlew
**gradlew.bat
**local.properties
.vscode
"#,
    )?;
    // currently , we don't have any config, just use a file to identifier the project root dir
    fs::write(outdir.join(RLA_CONFIG), "{}")?;
    Ok(())
}

#[instrument(skip_all, level = "debug", fields(dex=dex.file_name().unwrap().to_str().unwrap()))]
async fn task_baksmali(dex: PathBuf, smalis_dir: PathBuf, baksmali_jar: PathBuf) -> Result<String> {
    let dexname = dex.file_name().context("dex file no name")?;
    let outdir = smalis_dir.join(dexname);
    crate::cmd::baksmali(&dex, &outdir, &baksmali_jar)
}

#[instrument(skip_all, level = "debug")]
async fn task_unzip(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    let unpacked = outdir.join("unpacked");
    crate::zip::unzip(&apk, &unpacked)?;

    let smalis = outdir.join("smalis");
    fs::create_dir(&smalis)?;

    let baksmali_jar = BAKSMALI.release_binary(binarydir())?;
    let handles = walkdir::WalkDir::new(&unpacked)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .filter(|p| p.is_file() && p.extension().eq(&Some(OsStr::new("dex"))))
        .map(|dex| tokio::spawn(task_baksmali(dex, smalis.clone(), baksmali_jar.clone())))
        .collect::<Vec<_>>();
    for h in handles {
        h.await??;
    }
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
    if let Err(e) = crate::cmd::jadx_extract_src(&apk, &outdir.join("jadx-src")) {
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

pub(crate) async fn run(outdir: PathBuf, apk: PathBuf, no_jadx: bool, no_git: bool) -> Result<()> {
    // >> base.apk
    // >> unzip >> smali
    // >> git init
    // >> jadx
    // >> .gitginore, .rla.config.json
    // ====
    // >> git commit

    // parallel tasks begin
    let mut handles = vec![
        spawn(task_prepare_files(outdir.clone(), apk.clone())),
        spawn(task_unzip(outdir.clone(), apk.clone())),
    ];

    if !no_git {
        handles.push(spawn(task_git_init(outdir.clone())));
    }
    if !no_jadx {
        handles.push(spawn(task_jadx_reverse(outdir.clone(), apk.clone())));
    }
    for h in handles {
        h.await??;
    }

    if !no_git {
        task_git_commit(outdir, "Frist init project".to_string()).await;
    }
    Ok(())
}
