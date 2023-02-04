use std::path::Path;

use bevy::prelude::*;
use log::info;

use crate::{page::static_page::HttpAssetServeBundle, config::ServiceConfig};

use super::assets::SiteMapAsset;

#[derive(Component)]
pub struct SiteMapController {
    map: Handle<SiteMapAsset>,
}

pub(in super) fn site_map_reloader(cfg: Res<ServiceConfig>, mut asset_events: EventReader<AssetEvent<SiteMapAsset>>, controllers: Query<(Entity, &mut SiteMapController)>, mut commands: Commands, assets: Res<Assets<SiteMapAsset>>, asset_server: Res<AssetServer>) {
    if controllers.is_empty() {
        for i in &cfg.sitemaps {
            let asset = asset_server.load::<SiteMapAsset, &Path>(i);
            setup_sitemap(asset, &controllers, &mut commands, &assets, &asset_server);
        }
    }  
    
    for ev in asset_events.iter() {
        // If modified, build or rebuild a map with it.
        // Due to how hot-reload works, the handle id MUST be the same, and it must be loaded by the time we get this event.
        match ev {
            AssetEvent::Modified { handle } | AssetEvent::Created { handle } => {
                let mut h = handle.clone();
                h.make_strong(&assets);
                setup_sitemap(h, &controllers, &mut commands, &assets, &asset_server);
            }
            _ => (),
        }
    }
}

pub fn setup_sitemap(handle: Handle<SiteMapAsset>, controllers: &Query<(Entity, &mut SiteMapController)>, commands: &mut Commands, assets: &Res<Assets<SiteMapAsset>>, asset_server: &Res<AssetServer>) {
    let handle = handle.clone();

    let curr_controller = 'block: {
        for (e, controller) in controllers.iter() {
            if controller.map == handle {
                break 'block Some(e);
            }
        }

        None
    };

    let ecommands = {
        if let Some(e) = curr_controller {
            let mut ecmds = commands.entity(e);
            ecmds.despawn_descendants();
            Some(ecmds)
        } else {
            None
        }
    };

    // this is HAIRY logic, christ.
    let mut ecommands = if let None = ecommands {
        let controller = SiteMapController {
            map: handle.clone(),
        };
    
        info!("Spawned a new sitemap controller.");
        commands.spawn((controller, Name::new("SiteMap Controller")))
    } else { ecommands.unwrap() };
    
    
    let map = assets.get(&handle);
    if let Some(map_real) = map {
        info!("Loading a new site map, this will map the following pages: {:?}", map_real.mapping);
    } else {
        info!("Doing prelim loading of site map. Load state: {:?}", asset_server.get_load_state(handle));
        
        return;
    }

    ecommands.with_children(|b| {
        if let Some(map_real) = map {
            for (path, asset) in &map_real.mapping {
                let bundle = HttpAssetServeBundle::new(asset, path.clone(), &asset_server);

                if let Err(e) = bundle {
                    error!("Failed to set up {path:?} with {asset:?} due to {}",e);
                    continue;
                }

                b.spawn(bundle.unwrap());
            }
        }
    });
}
