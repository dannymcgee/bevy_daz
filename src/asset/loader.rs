use std::collections::VecDeque;

use anyhow::anyhow;
use bevy::{
	asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
	prelude::*,
	render::mesh::{Mesh, VertexAttributeValues},
	utils::{
		hashbrown::{HashMap, HashSet},
		BoxedFuture,
	},
};
use daz_asset_types::{ChannelsAsVec3, Daz, Geometry, Modifier, Node, NodeType};
use regex::{Captures, Regex};
use serde_json as json;

use crate::asset::{DazAsset, DazMesh, DazNode, DazPrimitive, DazUvSet};

#[derive(Clone, Copy, Debug, Default)]
pub struct DazAssetLoader;

impl AssetLoader for DazAssetLoader {
	type Asset = DazAsset;
	type Settings = (); // TODO
	type Error = anyhow::Error;

	fn load<'a>(
		&'a self,
		reader: &'a mut Reader,
		_: &'a Self::Settings,
		cx: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
		Box::pin(async move {
			let mut s = String::new();
			reader
				.read_to_string(&mut s)
				.await
				.map_err(|err| anyhow!("{err}"))?;

			let mut daz = json::from_str::<Daz>(&s).map_err(|err| anyhow!("{err}"))?;

			let raw_nodes = daz.node_library.take().unwrap_or_default();
			let TempNodesData {
				mut nodes,
				node_indices,
				mut children,
			} = process_nodes(&raw_nodes);

			let geo_lib = daz.geometry_library.take().unwrap_or_default();
			let mut meshes = process_geometries(cx, geo_lib).await?;

			let mut mods_lib = daz.modifier_library.take().unwrap_or_default();
			process_skins(&mut meshes, &raw_nodes, &mut mods_lib);

			let meshes = finish_meshes(cx, meshes, &mut nodes, &node_indices);
			let nodes = finish_nodes(cx, nodes, &mut children);

			let uv_sets = daz
				.uv_set_library
				.take()
				.unwrap_or_default()
				.into_iter()
				.map(|uv_set| {
					(uv_set.id, DazUvSet {
						vertex_count: uv_set.vertex_count,
						uvs: uv_set.uvs.values,
						polygon_vertex_indices: uv_set.polygon_vertex_indices,
					})
				})
				.collect();

			Ok(DazAsset {
				meshes,
				nodes,
				materials: Default::default(), // TODO
				uv_sets,
			})
		})
	}

	fn extensions(&self) -> &[&str] {
		&["dsf"]
	}
}

struct TempNodesData {
	/// Tuples of the original `node.id` and their corresponding [DazNode] asset
	nodes: Vec<(String, DazNode)>,
	/// Map of node IDs to their corresponding index in this structure's `nodes`
	/// field
	node_indices: HashMap<String, usize>,
	/// Map of node IDs to arrays of their children, represented as indices into
	/// this structure's `nodes` field
	children: HashMap<String, Vec<usize>>,
}

fn process_nodes(raw_nodes: &[Node]) -> TempNodesData {
	let mut nodes: Vec<(String, DazNode)> = Vec::with_capacity(raw_nodes.len());
	let mut node_indices: HashMap<String, usize> = HashMap::with_capacity(raw_nodes.len());
	let mut children: HashMap<String, Vec<usize>> = HashMap::with_capacity(raw_nodes.len());

	for raw_node in raw_nodes.iter() {
		let id = raw_node.id.clone();
		let name = raw_node.name.clone();
		let type_ = raw_node.r#type;
		let idx = nodes.len();
		node_indices.insert(id.clone(), idx);

		let root_transform = GlobalTransform::from(Transform {
			translation: raw_node.center_point.as_vec3() * 0.01,
			rotation: raw_node.orientation_quat(),
			scale: Vec3::splat(1.), // TODO
		});

		let end_point = root_transform
			.affine()
			.inverse()
			.transform_point(raw_node.end_point.as_vec3() * 0.01);

		let parent_id = raw_node.parent.as_ref().map(|selector| &selector[1..]);
		let parent_idx = parent_id.and_then(|id| node_indices.get(id).copied());
		let parent_root_transform = parent_idx
			.and_then(|idx| nodes.get(idx))
			.map(|(_, parent_node)| parent_node.root_transform)
			.unwrap_or_default();

		let transform = root_transform.reparented_to(&parent_root_transform);

		nodes.push((id.clone(), DazNode {
			id,
			name,
			type_,
			mesh: None,
			root_transform,
			transform,
			end_point,
			parent: parent_id.map(|id| id.to_owned()),
			children: vec![],
		}));

		if let Some(parent_id) = parent_id {
			if let Some(siblings) = children.get_mut(parent_id) {
				siblings.push(idx);
			} else {
				children.insert_unique_unchecked(parent_id.to_owned(), vec![idx]);
			}
		}
	}

	TempNodesData {
		nodes,
		node_indices,
		children,
	}
}

struct TempMeshData {
	name: Option<String>,
	mesh: Mesh,
	vertex_count: usize,
	joints: Vec<String>,
}

async fn process_geometries(
	cx: &mut LoadContext<'_>,
	geo_lib: Vec<Geometry>,
) -> anyhow::Result<HashMap<String, TempMeshData>> {
	let mut result: HashMap<String, TempMeshData> = HashMap::with_capacity(geo_lib.len());

	for raw_geo in geo_lib {
		let id = raw_geo.id.clone();
		let name = raw_geo.name.clone();
		let vertex_count = raw_geo.vertices.count;
		let default_uv_set_uri = raw_geo.default_uv_set.as_ref().cloned();
		let mut mesh = Mesh::from(raw_geo);

		if let Some(uri) = default_uv_set_uri {
			// TODO: Should probably break this URI parsing out into a
			//       separate function and be more judicious about caching
			//       the compiled regular expressions.
			let rel_path = uri.strip_prefix('/').unwrap_or(&uri);
			let unicode_re = Regex::new(r"%([0-9]{2})").unwrap();
			let decoded = unicode_re.replace_all(rel_path, |captures: &Captures| {
				let code_point = captures.get(1).unwrap().as_str();
				let code_point = u32::from_str_radix(code_point, 16).unwrap();
				char::from_u32(code_point).unwrap().to_string()
			});

			let fragment_re = Regex::new(r"#(.+)").unwrap();
			let target_id = fragment_re
				.captures(decoded.as_ref())
				.unwrap()
				.get(1)
				.unwrap()
				.as_str();

			let untyped = cx
				.load_direct(format!("daz://{decoded}"))
				.await
				.map_err(|err| anyhow!("{err}"))?;

			let daz_asset = untyped.get::<DazAsset>().unwrap();
			let uv_set = daz_asset.uv_sets.get(target_id).unwrap();
			// TODO: We're not currently dealing correctly with alternate
			//       coords for vertices at UV boundaries. I'm a little bit
			//       confused about the representation of the
			//       `polygon_vertex_indices` array, so this will require
			//       some additional research.
			let uvs = uv_set.uvs[..vertex_count].to_vec();

			mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
		}

		result.insert(id, TempMeshData {
			name,
			mesh,
			vertex_count,
			joints: vec![],
		});
	}

	Ok(result)
}

fn process_skins(
	meshes: &mut HashMap<String, TempMeshData>,
	raw_nodes: &[Node],
	mods_lib: &mut [Modifier],
) {
	let joint_ids = raw_nodes
		.iter()
		.filter_map(|node| {
			if node.r#type == NodeType::Bone {
				Some(&node.id[..])
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	let joint_indices = joint_ids
		.iter()
		.enumerate()
		.map(|(idx, id)| (*id, idx))
		.collect::<HashMap<_, _>>();

	for skin in mods_lib.iter_mut().filter_map(|m| m.skin.take()) {
		let vert_count = skin.vertex_count;
		let mesh_id = &skin.geometry[1..];
		if !meshes.contains_key(mesh_id) || vert_count != meshes[mesh_id].vertex_count {
			error!("Geometry '{mesh_id}' not found for skin!");
			continue;
		}

		let Some(joints) = skin.joints else {
			warn!("No joints found for '{mesh_id}' skin!");
			continue;
		};

		let mut vert_joints: Vec<[Option<usize>; 4]> = vec![[None, None, None, None]; vert_count];
		let mut vert_weights: Vec<[Option<f32>; 4]> = vec![[None, None, None, None]; vert_count];

		let mut excess_weights_by_vert = HashMap::<usize, usize>::default();

		for joint in joints {
			let Some(joint_idx) = joint_indices.get(&joint.node[1..]).copied() else {
				error!("Failed to find joint index for node '{}'", joint.node);
				continue;
			};
			let Some(node_weights) = joint.node_weights else {
				error!("No node weights for joint '{}'", joint.node);
				continue;
			};

			for (vert_idx, weight) in node_weights.values {
				let Some((buf_idx, vert_joint)) = (match vert_joints[vert_idx]
					.iter_mut()
					.enumerate()
					.find(|(_, vert_joint)| vert_joint.is_none())
				{
					None => {
						if excess_weights_by_vert.contains_key(&vert_idx) {
							*excess_weights_by_vert.get_mut(&vert_idx).unwrap() += 1;
						} else {
							excess_weights_by_vert.insert_unique_unchecked(vert_idx, 1);
						}

						let lowest_weight = vert_weights[vert_idx].iter().enumerate().fold(
							(usize::MAX, f32::MAX),
							|accum, current| {
								if current.1.unwrap() < accum.1 {
									(current.0, current.1.unwrap())
								} else {
									accum
								}
							},
						);

						if lowest_weight.1 < weight {
							Some((lowest_weight.0, &mut vert_joints[vert_idx][lowest_weight.0]))
						} else {
							None
						}
					}
					some => some,
				}) else {
					continue;
				};

				*vert_joint = Some(joint_idx);
				vert_weights[vert_idx][buf_idx] = Some(weight);
			}
		}

		// for (vert_idx, excess_weights) in excess_weights_by_vert {
		// 	warn!(
		// 		"Attempted to assign {} joint weights to vertex {vert_idx}, \
		// 		but a maximum of 4 are supported.",
		// 		excess_weights + 4,
		// 	)
		// }

		let vert_joints = vert_joints
			.into_iter()
			.map(|[a, b, c, d]| {
				let a = a.unwrap_or_default();
				let b = b.unwrap_or_default();
				let c = c.unwrap_or_default();
				let d = d.unwrap_or_default();

				[a as u16, b as u16, c as u16, d as u16]
			})
			.collect::<Vec<_>>();

		let vert_weights = vert_weights
			.into_iter()
			.map(|[a, b, c, d]| {
				let a = a.unwrap_or_default();
				let b = b.unwrap_or_default();
				let c = c.unwrap_or_default();
				let d = d.unwrap_or_default();

				let sum = a + b + c + d;
				if sum.abs() <= f32::EPSILON {
					Vec4::new(0., 0., 0., 0.)
				} else {
					Vec4::new(a, b, c, d) / sum
				}
			})
			.collect::<Vec<_>>();

		let mesh_data = meshes.get_mut(mesh_id).unwrap();

		mesh_data.joints = joint_ids.iter().copied().map(|id| id.to_owned()).collect();

		mesh_data.mesh.insert_attribute(
			Mesh::ATTRIBUTE_JOINT_INDEX,
			VertexAttributeValues::Uint16x4(vert_joints),
		);

		mesh_data
			.mesh
			.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vert_weights);
	}
}

fn finish_meshes(
	cx: &mut LoadContext<'_>,
	meshes: impl IntoIterator<Item = (String, TempMeshData)>,
	nodes: &mut [(String, DazNode)],
	node_indices: &HashMap<String, usize>,
) -> HashMap<String, Handle<DazMesh>> {
	let mut result = HashMap::default();

	for (idx, (id, mesh_data)) in meshes.into_iter().enumerate() {
		let mesh_name = mesh_data.name;

		let daz_prim = DazPrimitive {
			mesh: cx.add_labeled_asset(format!("{id}/Primitive{idx}"), mesh_data.mesh),
			material: None,
		};

		let mesh_handle = cx.add_labeled_asset(id.clone(), DazMesh {
			primitives: vec![daz_prim],
			joints: mesh_data.joints,
		});

		result.insert(id, mesh_handle.clone());

		if let Some(mesh_name) = mesh_name {
			if let Some(&parent_idx) = node_indices.get(&mesh_name) {
				let (_, node) = nodes.get_mut(parent_idx).unwrap();
				node.mesh = Some(mesh_handle);
			}
		}
	}

	result
}

fn finish_nodes(
	cx: &mut LoadContext<'_>,
	nodes: impl IntoIterator<Item = (String, DazNode)>,
	children: &mut HashMap<String, Vec<usize>>,
) -> HashMap<String, Handle<DazNode>> {
	let resolved_nodes = resolve_node_hierarchy(
		nodes
			.into_iter()
			.map(|(id, node)| {
				let children = children.remove(&id).unwrap_or_default();
				(id, node, children)
			})
			.collect(),
	);

	resolved_nodes
		.into_iter()
		.map(|(id, node)| (id.clone(), cx.add_labeled_asset(id, node)))
		.collect()
}

fn resolve_node_hierarchy(nodes: Vec<(String, DazNode, Vec<usize>)>) -> Vec<(String, DazNode)> {
	let mut has_errored = false;
	let mut empty_children = VecDeque::new();
	let mut parents = vec![None; nodes.len()];
	let mut unprocessed = nodes
		.into_iter()
		.enumerate()
		.map(|(idx, (id, node, children))| {
			for child in children.iter().copied() {
				if let Some(parent) = parents.get_mut(child) {
					*parent = Some(idx);
				} else if !has_errored {
					has_errored = true;
					error!("Unexpected child in DazNode: {child}");
				}
			}

			let children = children.into_iter().collect::<HashSet<_>>();
			if children.is_empty() {
				empty_children.push_back(idx);
			}

			(idx, (id, node, children))
		})
		.collect::<HashMap<_, _>>();

	let mut nodes = HashMap::<usize, (String, DazNode)>::new();
	while let Some(idx) = empty_children.pop_front() {
		let (id, node, _) = unprocessed.remove(&idx).unwrap();
		nodes.insert(idx, (id, node));

		if let Some(parent_idx) = parents[idx] {
			let (_, parent, siblings) = unprocessed.get_mut(&parent_idx).unwrap();
			assert!(siblings.remove(&idx));

			if let Some((_, child)) = nodes.get(&idx) {
				parent.children.push(child.clone());
			}

			if siblings.is_empty() {
				empty_children.push_back(parent_idx);
			}
		}
	}

	if !unprocessed.is_empty() {
		error!("Expected DazAsset to be a tree!");
	}

	let mut nodes = nodes.into_iter().collect::<Vec<_>>();
	nodes.sort_by_key(|(i, _)| *i);
	nodes.into_iter().map(|(_, tuple)| tuple).collect()
}
