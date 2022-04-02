use anyhow::Result;
use argh::FromArgs;

use crate::{
    deps::{APK_SIGNER, BAKSMALI, SMALI},
    jar,
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
}

#[derive(FromArgs)]
/// "java -jar apksigner ..."
#[argh(subcommand, name = "apksigner")]
struct ApkSigner {
    /// arguments for apksigner
    #[argh(positional)]
    args: Vec<String>,
}

#[derive(FromArgs)]
/// "java -jar smali ..."
#[argh(subcommand, name = "smali")]
struct Smali {
    /// arguments for smali
    #[argh(positional)]
    args: Vec<String>,
}

#[derive(FromArgs)]
/// "java -jar baksmali ..."
#[argh(subcommand, name = "baksmali")]
struct BakSmali {
    /// arguments for baksmali
    #[argh(positional)]
    args: Vec<String>,
}

#[derive(FromArgs)]
/// sign apk use a default debug keystore
#[argh(subcommand, name = "sign")]
struct Sign {
    /// file to sign
    #[argh(positional)]
    file: String,
}

impl Cli {}

pub(crate) fn run() -> Result<()> {
    let cli: Cli = argh::from_env();
    crate::log::init_logger(cli.verbose);

    match cli.nested {
        SubCommands::ApkSigner(ApkSigner { args }) => jar::run_jar(APK_SIGNER, &args),
        SubCommands::Smali(Smali { args }) => jar::run_jar(SMALI, &args),
        SubCommands::BakSmali(BakSmali { args }) => jar::run_jar(BAKSMALI, &args),
        SubCommands::Sign(Sign { file }) => jar::debugsign(&file),
    }
}
