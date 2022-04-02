use std::process::exit;

use anyhow::Result;
use argh::FromArgs;

use crate::{
    deps::{APK_SIGNER, BAKSMALI, SMALI},
    jar, reverse,
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
/// sign apk use a default debug keystore
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
struct Pack {}

// Some commands will print their error to stderr,
// we don't need to print it aigan in non-verbose mode
macro_rules! ok {
    ($r:expr) => {
        match $r {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("{e}");
                Ok(())
            }
        }
    };
}

// argh doesn't support forward all arguments to another command,
// so we handle it first manually.
// note that, for these commands, debugger logger can't be enabled
fn run_forward_command() -> bool {
    let subcmd = match std::env::args().nth(1) {
        None => return false,
        Some(s) => s,
    };
    let args = std::env::args().skip(2).collect::<Vec<_>>();
    let _ = match subcmd.as_str() {
        "smali" => jar::run_jar(SMALI, &args),
        "baksmali" => jar::run_jar(BAKSMALI, &args),
        "apksigner" => jar::run_jar(APK_SIGNER, &args),
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
        SubCommands::Sign(Sign { file }) => ok!(jar::debugsign(&file)),
        SubCommands::Unpack(Unpack { file }) => reverse::unpack_apk(&file),
        SubCommands::Pack(_) => reverse::pack_apk(),
        _ => {
            eprintln!("unhandled command, internal bug!");
            exit(-1);
        }
    }
}
