use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| tokio::runtime::Builder::new_multi_thread().build().unwrap());

pub fn rt() -> &'static Runtime {
    &*RUNTIME
}
