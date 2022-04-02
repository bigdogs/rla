use std::path::Path;

use once_cell::sync::Lazy;
use tempfile::{tempdir, TempDir};

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
