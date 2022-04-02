use std::{
    ffi::OsStr,
    fs,
    io::{self, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{format_err, Context, Result};
use tokio::spawn;
use tracing::{debug, error, info, instrument};

use crate::{deps::BAKSMALI, dir::binarydir};

use super::RLA_CONFIG;

/// run command, return console message
fn run_cmd(mut cmd: Command) -> Result<String> {
    debug!("{cmd:?}");

    let output = cmd.output().context("fail to run command")?;
    let mut msg = format!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        if !msg.is_empty() {
            msg.push('\n');
        }
        msg.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    match output.status.exit_ok() {
        Err(_) => Err(format_err!("exit code: {}, msg:\n{msg}", output.status)),
        Ok(_) => Ok(msg),
    }
}

fn git_init(workdir: &Path) -> Result<String> {
    let mut git = Command::new("git");
    git.current_dir(workdir)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("init");
    run_cmd(git)
}

fn git_commit(workdir: &Path, msg: &str) -> Result<String> {
    let mut git = Command::new("git");
    git.current_dir(workdir)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(&["commit", "-m", msg]);
    run_cmd(git)
}

fn git_add(workdir: &Path) -> Result<String> {
    let mut git = Command::new("git");
    git.current_dir(workdir)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(&["add", "."]);
    run_cmd(git)
}

// move to helper
fn unzip(apk: &Path, outdir: &Path) -> Result<()> {
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

fn jadx_reverse(outdir: &Path, apk: &Path) -> Result<String> {
    let mut jadx = Command::new("jadx");
    jadx.stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("-e")
        .arg(apk)
        .arg("-d")
        .arg(outdir.join("jadx-src"));
    run_cmd(jadx)
}

#[instrument(skip_all)]
async fn task_prepare_files(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    let bak = outdir.join("bak.apk");
    fs::copy(&apk, &bak)?;

    fs::write(
        outdir.join(".gitignore"),
        r#"output
**.DS_Store
"#,
    )?;
    // currently , we don't have any config, just use a file to identifier the project root dir
    fs::write(outdir.join(RLA_CONFIG), "{}")?;
    Ok(())
}

#[instrument(skip_all, fields(dex=dex.file_name().unwrap().to_str().unwrap()))]
async fn task_baksmali(dex: PathBuf, smalis_dir: PathBuf, baksmali_jar: PathBuf) -> Result<String> {
    let dexname = dex.file_name().context("dex file no name")?;
    let outdir = smalis_dir.join(dexname);

    let mut baksmali = Command::new("java");
    baksmali
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("-jar")
        .arg(baksmali_jar)
        .arg("d")
        .arg(dex)
        .arg("-o")
        .arg(outdir);
    run_cmd(baksmali)
}

#[instrument(skip_all)]
async fn task_unzip(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    let unpacked = outdir.join("unpacked");
    unzip(&apk, &unpacked)?;

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

#[instrument(skip_all)]
async fn task_git_init(outdir: PathBuf) -> Result<()> {
    if let Err(e) = git_init(&outdir) {
        // Notice user that git is not available, but not fail the procedure
        error!("git commit error {e}");
    }
    Ok(())
}

#[instrument(skip_all)]
async fn task_jadx_reverse(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    if let Err(e) = jadx_reverse(&outdir, &apk) {
        error!("jadx error:  {e}");
    }
    Ok(())
}

#[instrument(skip_all)]
async fn task_git_commit(outdir: PathBuf, msg: String) {
    if let Err(e) = git_add(&outdir).and_then(|_| git_commit(&outdir, &msg)) {
        error!("git commit error {e}");
    }
}

pub(crate) async fn run(outdir: PathBuf, apk: PathBuf) -> Result<()> {
    // >> base.apk
    // >> unzip >> smali
    // >> git init
    // >> jadx
    // >> .gitginore, .rla.config.json
    // ====
    // >> git commit

    // try_join will schedule our tasks in sequence, use spawn
    let handles = vec![
        spawn(task_prepare_files(outdir.clone(), apk.clone())),
        spawn(task_unzip(outdir.clone(), apk.clone())),
        spawn(task_git_init(outdir.clone())),
        spawn(task_jadx_reverse(outdir.clone(), apk.clone())),
    ];
    for h in handles {
        h.await??;
    }
    task_git_commit(outdir, "Frist init project".to_string()).await;
    Ok(())
}
