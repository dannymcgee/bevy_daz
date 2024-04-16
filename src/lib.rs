use bevy::app::{PluginGroup, PluginGroupBuilder};

mod asset;
mod io;
mod spawning;

pub use crate::{
	asset::{DazAsset, DazAssetTypesPlugin, DazMesh, DazNode, DazPrimitive, DazUvSet},
	io::{DazAssetReader, DazAssetSourcePlugin},
	spawning::{DazBone, DazFigure, DazSpawningPlugin},
};
pub use daz_asset_types::NodeType;

pub struct DazPlugins;

impl PluginGroup for DazPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(DazAssetTypesPlugin)
			.add(DazSpawningPlugin)
	}
}
