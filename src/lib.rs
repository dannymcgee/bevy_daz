use bevy::app::{PluginGroup, PluginGroupBuilder};

mod asset;
mod io;
mod runtime;
mod spawning;

pub use crate::{
	asset::{DazAsset, DazAssetTypesPlugin, DazMesh, DazNode, DazPrimitive, DazUvSet},
	io::{DazAssetReader, DazAssetSourcePlugin},
	runtime::DazRuntimePlugin,
	spawning::{DazBone, DazFigure, DazSpawningPlugin},
};
pub use bevy_dqskinning::DualQuat;
pub use daz_asset_types::NodeType;

pub struct DazPlugins;

impl PluginGroup for DazPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(DazAssetTypesPlugin)
			.add(DazSpawningPlugin)
			.add(DazRuntimePlugin)
	}
}
