use bevy::{
    app::{ScheduleRunnerPlugin, ScheduleRunnerSettings},
    prelude::*,
};
use log::{debug, error, info, trace, warn};
use std::{error::Error, time::Duration};
mod custtcpstream;
mod http;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;
    App::new()
        .add_plugin(CorePlugin::default())
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_nanos(0)))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(http::HttpRequestPlugin::default())
        .run();
    Ok(())
}
