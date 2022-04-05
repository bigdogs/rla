use std::process::exit;

use anyhow::Result;
use argh::FromArgs;

use crate::{
    deps::{APK_SIGNER, BAKSMALI, SMALI},
    reverse,
};

#[derive(FromArgs)]
/// Top-level command.
struct Cli {
    #[argh(subcommand)]
    nested: SubCommands,
    /// enable verbose mode
    #[argh(switch, short = 'v')]
    verbose: bool,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommands {
    ApkSigner(ApkSigner),
    Sign(Sign),
    Smali(Smali),
    BakSmali(BakSmali),
    Unpack(Unpack),
    Pack(Pack),
}

#[derive(FromArgs)]
/// "java -jar apksigner ..."
#[argh(subcommand, name = "apksigner")]
struct ApkSigner {}

#[derive(FromArgs)]
/// "java -jar smali ..."
#[argh(subcommand, name = "smali")]
struct Smali {}

#[derive(FromArgs)]
/// "java -jar baksmali ..."
#[argh(subcommand, name = "baksmali")]
struct BakSmali {}

#[derive(FromArgs)]
/// sign apk with a default debug keystore
#[argh(subcommand, name = "sign")]
struct Sign {
    /// file to sign
    #[argh(positional)]
    file: String,
}

#[derive(FromArgs)]
/// init reverse project for the apk file
#[argh(subcommand, name = "unpack")]
struct Unpack {
    /// apk file to sign
    #[argh(positional)]
    file: String,
}

#[derive(FromArgs)]
/// package revere project to make a new apk
#[argh(subcommand, name = "pack")]
struct Pack {
    /// directory of project
    #[argh(option, short = 'd')]
    dir: Option<String>,
}

// `argh` doesn't support forward all arguments to another command,
// so we handle it manually first.
// Tracing will not enabled for these commands
fn run_forward_command() -> bool {
    let subcmd = match std::env::args().nth(1) {
        None => return false,
        Some(s) => s,
    };
    let args = std::env::args().skip(2).collect::<Vec<_>>();
    let _ = match subcmd.as_str() {
        "smali" => crate::cmd::run_jar(SMALI, &args),
        "baksmali" => crate::cmd::run_jar(BAKSMALI, &args),
        "apksigner" => crate::cmd::run_jar(APK_SIGNER, &args),
        _ => return false,
    };
    true
}

pub(crate) fn run() -> Result<()> {
    if run_forward_command() {
        return Ok(());
    }

    let cli: Cli = argh::from_env();
    crate::log::init_logger(cli.verbose);
    match cli.nested {
        SubCommands::Sign(Sign { file }) => crate::cmd::debugsign(file.as_ref()),
        SubCommands::Unpack(Unpack { file }) => reverse::unpack_apk(&file),
        SubCommands::Pack(Pack { dir }) => reverse::pack_apk(dir),
        _ => {
            eprintln!("unhandled command, internal bug!");
            exit(-1);
        }
    }
}
