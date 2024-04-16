use bevy::{ecs::entity::EntityHashMap, prelude::*, utils::HashMap};

use crate::{DazAsset, DazBone, DazFigure};

pub struct DazRuntimePlugin;

impl Plugin for DazRuntimePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PostUpdate, auto_follow_parent_skeletons);
	}
}

/// Daz "figures" are commonly split into separate skinned meshes -- one for the
/// main body, one for the eyes, one for the mouth, etc. Each of these pieces
/// have their own skeletons, which are at least partially identical to the
/// "main" figure skeleton.
///
/// This system updates the transforms of the "child" skeleton joints to match
/// the corresponding joints of the "parent" skeleton.
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
