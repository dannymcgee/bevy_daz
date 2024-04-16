use std::collections::VecDeque;

use bevy::{
	ecs::entity::{EntityHashMap, EntityHashSet},
	math::Affine3A,
	prelude::*,
	render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
	utils::HashMap,
};

use crate::{DazAsset, DazMesh, DazNode, NodeType};

pub struct DazSpawningPlugin;

impl Plugin for DazSpawningPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(DazSpawner::default());

		app.register_type::<DazFigure>();
		app.register_type::<DazBone>();

		app.add_systems(Update, (queue_asset_spawns, spawn_daz_assets));
		app.add_systems(PostUpdate, auto_follow_parent_skeletons);
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
	pub inverse_bindpose: Affine3A,
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
	mut ra_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
	mut ra_standard_mats: ResMut<Assets<StandardMaterial>>,
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
						inverse_bindpose: node.root_transform.affine().inverse(),
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
						// Note: Could do `root_transform.compute_matrix().inverse()`,
						//       but Affine3As are significantly cheaper to invert
						.map(|id| Mat4::from(spawned_nodes[id].root_transform.affine().inverse()))
						.collect::<Vec<_>>();

					let handle = ra_inverse_bindposes.add(inverse_bindposes);

					Some(SkinnedMesh {
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
								ra_standard_mats.add(StandardMaterial {
									base_color: Color::hex("AAAAAA").unwrap(),
									metallic: 0.,
									perceptual_roughness: 0.55,
									reflectance: 0.45,
									..default()
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

fn auto_follow_parent_skeletons(
	q_figures: Query<Entity, With<DazFigure>>,
	q_children: Query<&Children>,
	q_other_asset_roots: Query<Entity, (With<Handle<DazAsset>>, Without<DazFigure>)>,
	mut q_joints: Query<(&Name, &mut Transform), With<DazBone>>,
) {
	let follower_joint_sets = q_other_asset_roots
		.iter()
		.map(|root| {
			(
				root,
				q_children
					.iter_descendants(root)
					.filter_map(|desc| {
						q_joints
							.get(desc)
							.ok()
							.map(|(name, _)| (name.to_string(), desc))
					})
					.collect::<HashMap<_, _>>(),
			)
		})
		.collect::<EntityHashMap<_>>();

	let parent_joint_sets = q_figures
		.iter()
		.filter_map(|root| {
			let Ok(children) = q_children.get(root) else {
				return None;
			};

			let grandchildren = children
				.iter()
				.filter_map(|&child| q_children.get(child).ok())
				.flatten();

			grandchildren.copied().find_map(|grandchild| {
				if q_joints.contains(grandchild) {
					Some((root, grandchild))
				} else {
					None
				}
			})
		})
		.map(|(root, root_joint)| {
			(
				root,
				std::iter::once(root_joint)
					.chain(
						q_children
							.iter_descendants(root_joint)
							.filter(|&ent| q_joints.contains(ent)),
					)
					.collect::<Vec<_>>(),
			)
		})
		.collect::<EntityHashMap<_>>();

	for parent in q_figures.iter() {
		let Some(parent_joints) = parent_joint_sets.get(&parent) else {
			continue;
		};
		let Ok(children) = q_children.get(parent) else {
			continue;
		};
		let parent_joints = parent_joints
			.iter()
			.copied()
			.map(|joint_ent| {
				let (name, xform) = q_joints.get(joint_ent).unwrap();
				(name.to_string(), *xform)
			})
			.collect::<Vec<_>>();

		for child in children.iter() {
			let Some(follower_joints) = follower_joint_sets.get(child) else {
				continue;
			};

			for (name, leader_xform) in parent_joints.iter() {
				let Some(follower_joint) = follower_joints.get(name) else {
					continue;
				};

				let (_, mut follower_xform) = q_joints.get_mut(*follower_joint).unwrap();
				*follower_xform = *leader_xform;
			}
		}
	}
}
