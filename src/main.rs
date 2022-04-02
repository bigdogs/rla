#![feature(exit_status_error)]

mod cli;
mod deps;
mod dir;
mod jar;
mod log;
mod reverse;
mod runtime;

fn main() {
    if let Err(e) = cli::run() {
        tracing::error!("{e:?}");
    }
}
