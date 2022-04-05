mod shell;

use std::process::{Command, Stdio};

use anyhow::{format_err, Context, Result};
use tracing::debug;

pub(crate) use shell::{
    baksmali, debugsign, git_add, git_commit, git_init, jadx_extract_src, run_jar, smali,
};

fn cmd_to_string(cmd: &Command) -> String {
    let prog = cmd.get_program().to_str().unwrap_or("???");
    let mut vec = cmd
        .get_args()
        .into_iter()
        .map(|s| s.to_str().unwrap_or("???"))
        .collect::<Vec<_>>();
    if prog != vec[0] {
        vec.insert(0, prog);
    }
    vec.join(" ")
}

pub(crate) fn run(mut cmd: Command) -> Result<String> {
    let cmd_str = cmd_to_string(&cmd);
    debug!("{:?}", cmd_str);

    cmd.stderr(Stdio::piped()).stdout(Stdio::piped());
    let output = cmd
        .output()
        .with_context(|| format!("faile to run {cmd_str:?}"))?;

    let mut msg = format!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        if !msg.is_empty() {
            msg.push('\n');
        }
        msg.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    match output.status.exit_ok() {
        Err(_) => Err(format_err!(
            "{cmd_str:?} exit code: {}, msg:\n{msg}",
            output.status
        )),
        Ok(_) => Ok(msg),
    }
}

pub(crate) fn run_interit(cmd: Command) -> Result<()> {
    fn run(mut cmd: Command) -> Result<()> {
        Ok(cmd.status()?.exit_ok()?)
    }

    let cmd_str = cmd_to_string(&cmd);
    debug!("{:?}", cmd_str);
    run(cmd).with_context(|| format!("{:?} error", cmd_str))
}
