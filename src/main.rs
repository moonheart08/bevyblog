use bevy::{
    app::{ScheduleRunnerPlugin, ScheduleRunnerSettings},
    prelude::*,
};
use log::info;
use std::{error::Error, time::Duration, path::PathBuf, fs::File, io::Read};

use crate::{page::static_page::HttpFileServeBundle, config::ServiceConfig};
mod custtcpstream;
mod config;
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
    
    let config = {
        let mut data: String = String::new();
        let mut cfg_file = File::open("cfg.ron")?;
        cfg_file.read_to_string(&mut data)?;
        ron::from_str::<ServiceConfig>(&data)?
    };

    let mut app = App::new();
    app.add_plugin(CorePlugin::default())
        .add_plugin(AssetPlugin {
            // Tell the asset server to watch for asset changes on disk:
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(config)
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_micros(((1000.0/120.0) * 1000.0) as u64))) // I think only responding in 8ms periods is fine. This brings the CPU use from 100% to 0.1%. I'm not kidding.
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(http::HttpRequestPlugin::default())
        .add_plugin(page::HttpPageHandlerPlugin::default());
    let world = &mut app.world;
    let assets = world.get_resource::<AssetServer>().unwrap();
    let bundlea = HttpFileServeBundle::new(&PathBuf::from("index.html"), PathBuf::from("/"), &assets)?;
    let bundleb = HttpFileServeBundle::new(&PathBuf::from("index.html"), PathBuf::from("/index.html"), &assets)?;
    let bundlec = HttpFileServeBundle::new(&PathBuf::from("main.less"), PathBuf::from("/main.less"), &assets)?;
    world.spawn(bundlea);
    world.spawn(bundleb);
    world.spawn(bundlec);
    app.run();
    Ok(())
}
