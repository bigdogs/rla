/// debug+(verbose), error+(non-verbose)
pub(crate) fn init_logger(verbose: bool) {
    let level = {
        if verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::ERROR
        }
    };
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_max_level(level)
        .init();
}
