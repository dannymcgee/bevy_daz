use std::collections::VecDeque;

use bevy::{ecs::entity::EntityHashSet, pbr::ExtendedMaterial, prelude::*, utils::HashMap};
use bevy_dqskinning::{
	DqSkinnedMesh, DqSkinningPlugin, DqsInverseBindposes, DqsMaterialExt, DqsStandardMaterial,
	DualQuat,
};

use crate::{DazAsset, DazMesh, DazNode, NodeType};

pub struct DazSpawningPlugin;

impl Plugin for DazSpawningPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(DqSkinningPlugin);

		app.insert_resource(DazSpawner::default());

		app.register_type::<DazFigure>();
		app.register_type::<DazBone>();

		app.add_systems(Update, (queue_asset_spawns, spawn_daz_assets));
	}
}

#[derive(Resource, Clone, Debug, Default)]
struct DazSpawner {
	pending: EntityHashSet,
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct DazFigure;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct DazBone {
	pub end_point: Vec3,
	pub inverse_bindpose: DualQuat,
}

fn queue_asset_spawns(
	mut r_spawner: ResMut<DazSpawner>,
	q_added_daz_assets: Query<Entity, Added<Handle<DazAsset>>>,
) {
	for ent in q_added_daz_assets.iter() {
		r_spawner.pending.insert(ent);
	}
}

#[allow(clippy::too_many_arguments)]
fn spawn_daz_assets(
	mut cmd: Commands,
	r_assets: Res<AssetServer>,
	mut r_spawner: ResMut<DazSpawner>,
	ra_daz_assets: Res<Assets<DazAsset>>,
	ra_daz_nodes: Res<Assets<DazNode>>,
	ra_daz_meshes: Res<Assets<DazMesh>>,
	mut ra_inverse_bindposes: ResMut<Assets<DqsInverseBindposes>>,
	mut ra_dqstandard_mats: ResMut<Assets<DqsStandardMaterial>>,
	q_daz_assets: Query<&Handle<DazAsset>>,
	mut l_assets_to_spawn: Local<Vec<Entity>>,
) {
	l_assets_to_spawn.clear();

	let pending = r_spawner.pending.iter().copied().collect::<Vec<_>>();
	for entity in pending {
		if let Ok(handle) = q_daz_assets.get(entity) {
			if r_assets.is_loaded_with_dependencies(handle) {
				l_assets_to_spawn.push(entity);
				r_spawner.pending.remove(&entity);
			}
		} else {
			r_spawner.pending.remove(&entity);
		}
	}

	for asset_entity in l_assets_to_spawn.iter().copied() {
		let handle = q_daz_assets.get(asset_entity).unwrap();
		let asset = ra_daz_assets.get(handle).unwrap();

		let root_nodes = asset
			.nodes
			.iter()
			.filter_map(|(_, handle)| {
				ra_daz_nodes.get(handle).and_then(|node| {
					if node.parent.is_none() {
						Some(node)
					} else {
						None
					}
				})
			})
			.collect::<Vec<_>>();

		for root_node in root_nodes {
			let mut spawned_entities = HashMap::<String, Entity>::default();
			let mut spawned_nodes = HashMap::<String, &DazNode>::default();

			let root_entity = cmd
				.spawn((
					Name::new(root_node.id.clone()),
					SpatialBundle::from(root_node.transform),
				))
				.id();

			cmd.entity(asset_entity).add_child(root_entity);
			spawned_entities.insert(root_node.id.clone(), root_entity);
			spawned_nodes.insert(root_node.id.clone(), root_node);

			let mut nodes_to_spawn = VecDeque::new();
			nodes_to_spawn.extend(root_node.children.iter().map(|node| node.id.clone()));

			while let Some(next) = nodes_to_spawn.pop_front() {
				let handle = asset.nodes.get(&next).unwrap();
				let node = ra_daz_nodes.get(handle.clone()).unwrap();

				let entity = cmd
					.spawn((
						Name::new(node.name.clone()),
						SpatialBundle::from(node.transform),
					))
					.id();

				if node.type_ == NodeType::Bone {
					cmd.entity(entity).insert(DazBone {
						end_point: node.end_point,
						// TODO: Avoid doing this computation twice for each joint
						inverse_bindpose: node.root_transform.affine().inverse().into(),
					});
				}

				spawned_entities.insert(node.id.clone(), entity);
				spawned_nodes.insert(node.id.clone(), node);

				if let Some(parent_id) = node.parent.as_ref() {
					let parent_ent = spawned_entities[parent_id];
					cmd.entity(parent_ent).add_child(entity);
				}

				nodes_to_spawn.extend(node.children.iter().map(|node| node.id.clone()));
			}

			for (id, &node) in spawned_nodes.iter() {
				let Some(daz_mesh_handle) = node.mesh.as_ref() else {
					continue;
				};
				let node_entity = spawned_entities[id];

				let daz_mesh = ra_daz_meshes.get(daz_mesh_handle).unwrap();
				let joints = daz_mesh
					.joints
					.iter()
					.map(|id| spawned_entities[id])
					.collect::<Vec<_>>();

				let skinned_mesh = if !joints.is_empty() {
					let inverse_bindposes = daz_mesh
						.joints
						.iter()
						.map(|id| {
							// TODO: Avoid doing this computation twice for each joint
							DualQuat::from(spawned_nodes[id].root_transform.affine().inverse())
						})
						.collect::<Vec<_>>();

					let handle = ra_inverse_bindposes.add(inverse_bindposes);

					Some(DqSkinnedMesh {
						inverse_bindposes: handle,
						joints,
					})
				} else {
					None
				};

				for primitive in daz_mesh.primitives.iter() {
					let mesh_entity = cmd
						.spawn(MaterialMeshBundle {
							mesh: primitive.mesh.clone(),
							material: primitive.material.clone().unwrap_or_else(|| {
								ra_dqstandard_mats.add(ExtendedMaterial {
									base: StandardMaterial {
										base_color: Color::hex("AAAAAA").unwrap(),
										metallic: 0.,
										perceptual_roughness: 0.55,
										reflectance: 0.45,
										..default()
									},
									extension: DqsMaterialExt::default(),
								})
							}),
							..default()
						})
						.id();

					if let Some(skinned_mesh) = skinned_mesh.as_ref() {
						cmd.entity(mesh_entity).insert(skinned_mesh.clone());
					}

					cmd.entity(node_entity).add_child(mesh_entity);
				}
			}
		}
	}
}
