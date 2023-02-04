use std::path::PathBuf;

use bevy::prelude::Resource;
use serde::Deserialize;

#[derive(Debug, Deserialize, Resource, Clone)]
pub struct ServiceConfig {
    /// The site map assets to load for this site.
    pub sitemaps: Vec<PathBuf>,
    /// The bind addresses to utilize, in order of preference.
    /// # Example
    /// `["0.0.0.0:8080","0.0.0.0:8081"]`
    pub bind_addresses: Vec<String>,
}