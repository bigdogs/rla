use chrono::{DateTime, Local};
use tracing_subscriber::fmt::{format::FmtSpan, time::FormatTime};

struct Timer;

impl FormatTime for Timer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let local: DateTime<Local> = Local::now();
        write!(w, "{}", local.format("%Y-%m-%d %H:%M:%S.%3f"))
    }
}

/// debug+(verbose), error+(non-verbose)
pub(crate) fn init_logger(verbose: bool) {
    let level = {
        if verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    };
    let builder = tracing_subscriber::fmt()
        .with_file(verbose)
        .with_line_number(verbose)
        .with_level(verbose)
        .with_target(false)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_max_level(level)
        .with_timer(Timer);
    if verbose {
        builder.init();
    } else {
        builder.without_time().init();
    }
}
