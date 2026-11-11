use std::sync::Arc;

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing::{error, trace};

mod app;
mod errors;

pub use self::errors::{Error, Result};

fn main() -> Result<()> {
    let config = Arc::new(app::config::Config::parse());

    let level = match config.verbosity() {
        0 => LevelFilter::ERROR,
        1 => LevelFilter::WARN,
        2 => LevelFilter::INFO,
        3 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .init();

    trace!("Starting with {:?}", &config);
    let mut app = app::init(config)?;

    match app.run() {
        Err(Error::UserExit) => {
            trace!("Exit was requested by user, terminating");
            Ok(())
        }
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error occurred: {:?}", e);
            Err(e)
        }
    }
}
