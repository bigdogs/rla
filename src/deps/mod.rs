use anyhow::{Context, Result};
use std::{
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

/// copied from android sdk tools(30.0.3)
pub const APK_SIGNER: &Dep = &Dep {
    name: "apksigner.jar",
    bytes: include_bytes!("./apksigner.jar"),
};

/// copied from android sdk tools(30.0.3)
pub const DX: &Dep = &Dep {
    name: "dx.jar",
    bytes: include_bytes!("./dx.jar"),
};

/// android studio debug sign,
pub const DEBUG_STORE: &Dep = &Dep {
    name: "debug.keystore",
    bytes: include_bytes!("./debug.keystore"),
};

/// smali
/// https://bitbucket.org/JesusFreke/smali/downloads/
pub const SMALI: &Dep = &Dep {
    name: "smali",
    bytes: include_bytes!("./smali-2.5.2.jar"),
};

/// baksmali
/// https://bitbucket.org/JesusFreke/smali/downloads/
pub const BAKSMALI: &Dep = &Dep {
    name: "baksmali",
    bytes: include_bytes!("./baksmali-2.5.2.jar"),
};

pub struct Dep {
    pub name: &'static str,
    pub bytes: &'static [u8],
}

impl Dep {
    pub fn release_binary(&self, dir: &Path) -> Result<PathBuf> {
        let file = dir.join(self.name);
        let mut writer = BufWriter::new(fs::File::create(&file)?);
        writer
            .write_all(self.bytes)
            .context("release binary error")?;
        Ok(file)
    }
}
