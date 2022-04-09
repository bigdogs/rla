#![feature(exit_status_error)]

mod cli;
mod cmd;
mod core;
mod deps;
mod dir;
mod log;
mod runtime;
mod zip;

fn main() {
    let start = std::time::Instant::now();
    if let Err(e) = cli::run() {
        tracing::error!("{e:?}");
        std::process::exit(1);
    } else {
        tracing::debug!("task DONE:  {:.2?}", start.elapsed());
    }
}
