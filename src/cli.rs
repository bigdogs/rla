use std::process::exit;

use anyhow::Result;
use argh::FromArgs;

use crate::{
    core::{self, RlaConfig},
    deps::{APK_SIGNER, BAKSMALI, SMALI},
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
    JavaToSmali(JavaToSmali),
    SmaliToJava(SmaliToJava),
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
    /// apk file to unpack
    #[argh(positional)]
    file: String,
    /// disable jadx feature
    #[argh(switch)]
    no_jadx: bool,
    /// disable git feature
    #[argh(switch)]
    no_git: bool,
    /// unpack smali only
    #[argh(switch, long = "smali")]
    smali_only: bool,
    /// force override exists directory
    #[argh(switch)]
    force: bool,
}

impl Unpack {
    fn config(&self) -> RlaConfig {
        RlaConfig {
            smali_only: self.smali_only,
            git_enable: !self.no_git,
            jadx_enable: !self.no_jadx,
            force_override: self.force,
        }
    }
}

#[derive(FromArgs)]
/// package revere project to make a new apk
#[argh(subcommand, name = "pack")]
struct Pack {
    /// directory of project
    #[argh(option, short = 'd')]
    dir: Option<String>,
}

#[derive(FromArgs)]
/// compile java (to smali)
#[argh(subcommand, name = "cj")]
struct JavaToSmali {
    /// either a java file or a root dir for java files
    #[argh(positional)]
    path: String,
}

#[derive(FromArgs)]
/// compile smali (to java)
#[argh(subcommand, name = "cs")]
struct SmaliToJava {
    /// a smali file to compile
    #[argh(positional)]
    path: String,
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
        SubCommands::Unpack(c) => core::unpack_apk(&c.file, c.config()),
        SubCommands::Pack(Pack { dir }) => core::pack_apk(dir),
        SubCommands::JavaToSmali(JavaToSmali { path }) => core::java_to_smali(&path),
        SubCommands::SmaliToJava(SmaliToJava { path }) => core::smali_to_java(&path),
        _ => {
            eprintln!("unhandled command, internal bug!");
            exit(-1);
        }
    }
}
