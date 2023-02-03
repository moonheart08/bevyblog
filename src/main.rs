use bevy::{
    app::{ScheduleRunnerPlugin, ScheduleRunnerSettings},
    prelude::*,
};
use log::info;
use std::{error::Error, time::Duration, path::PathBuf};

use crate::page::static_page::HttpFileServeBundle;
mod custtcpstream;
mod http;
mod page;

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
    info!("Starting bevy application.");
    let mut app = App::new();
    app.add_plugin(CorePlugin::default())
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_nanos(0)))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(http::HttpRequestPlugin::default())
        .add_plugin(page::HttpPageHandlerPlugin::default());
    app.world.spawn(HttpFileServeBundle::new(&PathBuf::from("assets/index.html"), PathBuf::from("/"))?);
    app.world.spawn(HttpFileServeBundle::new(&PathBuf::from("assets/index.html"), PathBuf::from("/index.html"))?);
    app.world.spawn(HttpFileServeBundle::new(&PathBuf::from("assets/main.less"), PathBuf::from("/main.less"))?);
    app.run();
    Ok(())
}
