use std::path::PathBuf;

use bevy::asset::{AssetLoader, LoadedAsset};
use bevy::reflect::{TypeUuid};
use hyper::body::Bytes;
use serde::Deserialize;

#[derive(Debug, TypeUuid)]
#[uuid="5dadb1ea-82d0-40da-b864-596f8b2b40b7"]
pub struct WebFileAsset {
    pub data: Bytes,
}

#[derive(Debug, TypeUuid, Deserialize)]
#[uuid="c5c61281-cddb-47eb-9e76-d1c73a75105f"]
pub struct SiteMapAsset {
    pub mapping: Vec<(PathBuf, PathBuf)>
}

pub(in super) struct WebFileLoader();

impl AssetLoader for WebFileLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(WebFileAsset { data: Bytes::copy_from_slice(bytes) }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["html", "less", "css", "js", "png", "ico"]
    }
}

pub(in super) struct SiteMapLoader();

impl AssetLoader for SiteMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(ron::from_str::<SiteMapAsset>(&String::from_utf8_lossy(bytes))?));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}