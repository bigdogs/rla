#![feature(exit_status_error)]

use tracing::warn;

mod cli;
mod deps;
mod dir;
mod jar;
mod log;

fn main() {
    if let Err(e) = cli::run() {
        warn!("rla run error:  {e}");
    }
}
