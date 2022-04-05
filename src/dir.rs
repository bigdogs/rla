use std::{
    ffi::{OsStr, OsString},
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use once_cell::sync::Lazy;
use tempfile::{tempdir, TempDir, TempPath};

static BINARIES: Lazy<TempDir> = Lazy::new(|| {
    // assume that it will not create fail
    match tempdir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("tempdir create error {e}");
            std::process::exit(-1);
        }
    }
});

pub(crate) fn binarydir() -> &'static Path {
    BINARIES.path()
}

pub(crate) fn temppath<T: AsRef<OsStr>>(name: T) -> TempPath {
    static GEN: AtomicUsize = AtomicUsize::new(0);
    let id = GEN.fetch_add(1, Ordering::Relaxed);

    let mut base = OsString::new();
    base.push(format!("{id}_"));
    base.push(name.as_ref());

    let path = binarydir().join(name.as_ref());
    TempPath::from_path(path)
}
