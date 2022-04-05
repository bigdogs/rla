#![feature(exit_status_error)]

mod cli;
mod cmd;
mod deps;
mod dir;
mod log;
mod reverse;
mod runtime;
mod zip;

fn main() {
    let start = std::time::Instant::now();
    if let Err(e) = cli::run() {
        tracing::error!("{e:?}");
    } else {
        tracing::info!("DONE:  {:.2?}", start.elapsed());
    }
}
