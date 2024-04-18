use bevy::{prelude::*, utils::HashMap};
use bevy_dqskinning::DqsStandardMaterial;
use daz_asset_types::NodeType;

use self::loader::DazAssetLoader;

mod loader;

pub struct DazAssetTypesPlugin;

impl Plugin for DazAssetTypesPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<DazAsset>()
			.init_asset::<DazNode>()
			.init_asset::<DazMesh>()
			.init_asset::<DazPrimitive>()
			.init_asset::<DazUvSet>();

		app.register_asset_loader(DazAssetLoader);
	}
}

#[derive(Asset, Clone, Debug, TypePath)]
pub struct DazAsset {
	pub meshes: HashMap<String, Handle<DazMesh>>,
	pub nodes: HashMap<String, Handle<DazNode>>,
	// TODO
	pub materials: HashMap<String, Handle<StandardMaterial>>,
	pub uv_sets: HashMap<String, DazUvSet>,
}

#[derive(Asset, Clone, Debug, TypePath)]
pub struct DazNode {
	pub id: String,
	pub name: String,
	pub type_: NodeType,
	pub mesh: Option<Handle<DazMesh>>,
	pub root_transform: GlobalTransform,
	pub transform: Transform,
	pub parent: Option<String>,
	pub children: Vec<DazNode>,
	pub end_point: Vec3,
	// TODO: Formulas, rotation limits?
}

#[derive(Asset, Clone, Debug, TypePath)]
pub struct DazMesh {
	pub primitives: Vec<DazPrimitive>,
	pub joints: Vec<String>,
}

#[derive(Asset, Clone, Debug, TypePath)]
pub struct DazPrimitive {
	pub mesh: Handle<Mesh>,
	pub material: Option<Handle<DqsStandardMaterial>>,
}

#[derive(Asset, Clone, Debug, TypePath)]
pub struct DazUvSet {
	pub vertex_count: usize,
	pub uvs: Vec<Vec2>,
	pub polygon_vertex_indices: Option<Vec<[usize; 3]>>,
}
