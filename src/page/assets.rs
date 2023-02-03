use bevy::asset::{AssetLoader, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::{TypeUuid};

#[derive(Debug, TypeUuid)]
#[uuid="5dadb1ea-82d0-40da-b864-596f8b2b40b7"]
pub struct WebFileAsset {
    pub data: String,
}

pub(in super) struct WebFileLoader();

impl AssetLoader for WebFileLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(WebFileAsset { data: String::from_utf8_lossy(bytes).to_string() }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["html", "less", "css", "js"]
    }
}