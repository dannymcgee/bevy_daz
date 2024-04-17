use bevy::{
	asset::Assets,
	ecs::{
		component::Component,
		entity::{Entity, EntityHashMap},
		query::{With, Without},
		system::{Commands, Query, Res, ResMut, Resource},
	},
	prelude::{Deref, DerefMut},
	render::{
		batching::NoAutomaticBatching,
		render_resource::{BufferUsages, BufferVec},
		renderer::{RenderDevice, RenderQueue},
		view::ViewVisibility,
		Extract,
	},
	transform::components::GlobalTransform,
};

use crate::{DqSkinnedMesh, DqsInverseBindposes, DualQuat};

pub const JOINT_SIZE: usize = std::mem::size_of::<DualQuat>();
pub(crate) const JOINT_BUFFER_SIZE: usize = bevy::pbr::MAX_JOINTS * JOINT_SIZE;

#[derive(Component)]
pub struct DqSkinIndex {
	pub index: u32,
}

impl DqSkinIndex {
	/// Index to be in address space based on [`DqSkinUniform`] size.
	const fn new(start: usize) -> Self {
		Self {
			index: (start + std::mem::size_of::<DualQuat>()) as u32,
		}
	}
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct DqSkinIndices(EntityHashMap<DqSkinIndex>);

#[derive(Resource)]
pub struct DqSkinUniform {
	pub buffer: BufferVec<DualQuat>,
}

impl Default for DqSkinUniform {
	fn default() -> Self {
		Self {
			buffer: BufferVec::new(BufferUsages::UNIFORM),
		}
	}
}

pub fn prepare_skins(
	r_render_device: Res<RenderDevice>,
	r_render_queue: Res<RenderQueue>,
	mut r_uniform: ResMut<DqSkinUniform>,
) {
	if r_uniform.buffer.is_empty() {
		return;
	}

	let len = r_uniform.buffer.len();
	r_uniform.buffer.reserve(len, &r_render_device);
	r_uniform
		.buffer
		.write_buffer(&r_render_device, &r_render_queue);
}

// TODO: This was copy-pasted straight from `bevy_pbr::render::skin`, which
//       makes parts of this comment inaccurate (e.g., the uniform is declared
//       in the shader as an `array<mat2x4<f32>, 256u>`, though I'm not yet
//       confident that's correct)
//
// Notes on implementation:
// We define the uniform binding as an array<mat4x4<f32>, N> in the shader,
// where N is the maximum number of Mat4s we can fit in the uniform binding,
// which may be as little as 16kB or 64kB. But, we may not need all N.
// We may only need, for example, 10.
//
// If we used uniform buffers ‘normally’ then we would have to write a full
// binding of data for each dynamic offset binding, which is wasteful, makes
// the buffer much larger than it needs to be, and uses more memory bandwidth
// to transfer the data, which then costs frame time So @superdump came up
// with this design: just bind data at the specified offset and interpret
// the data at that offset as an array<T, N> regardless of what is there.
//
// So instead of writing N Mat4s when you only need 10, you write 10, and
// then pad up to the next dynamic offset alignment. Then write the next.
// And for the last dynamic offset binding, make sure there is a full binding
// of data after it so that the buffer is of size
// `last dynamic offset` + `array<mat4x4<f32>>`.
//
// Then when binding the first dynamic offset, the first 10 entries in the array
// are what you expect, but if you read the 11th you’re reading ‘invalid’ data
// which could be padding or could be from the next binding.
//
// In this way, we can pack ‘variable sized arrays’ into uniform buffer bindings
// which normally only support fixed size arrays. You just have to make sure
// in the shader that you only read the values that are valid for that binding.
pub fn extract_skins(
	mut r_skin_indices: ResMut<DqSkinIndices>,
	mut r_uniform: ResMut<DqSkinUniform>,
	q_skinned_mesh: Extract<Query<(Entity, &ViewVisibility, &DqSkinnedMesh)>>,
	ra_inverse_bindposes: Extract<Res<Assets<DqsInverseBindposes>>>,
	q_joints: Extract<Query<&GlobalTransform>>,
) {
	r_uniform.buffer.clear();
	r_skin_indices.clear();
	let mut last_start = 0;

	// PERF: This can be expensive, can we move this to prepare?
	for (entity, view_visibility, skin) in q_skinned_mesh.iter() {
		if !view_visibility.get() {
			continue;
		}
		let buffer = &mut r_uniform.buffer;
		let Some(inverse_bindposes) = ra_inverse_bindposes.get(&skin.inverse_bindposes) else {
			continue;
		};
		let start = buffer.len();

		let target = start + skin.joints.len().min(bevy::pbr::MAX_JOINTS);
		buffer.extend(
			q_joints
				.iter_many(&skin.joints)
				.zip(inverse_bindposes.iter())
				.take(bevy::pbr::MAX_JOINTS)
				.map(|(joint, inverse_bindpose)| DualQuat::from(*joint) * *inverse_bindpose),
		);

		// iter_many will skip any failed fetches. This will cause it to assign
		// the wrong bones, so just bail by truncating to the start.
		if buffer.len() != target {
			buffer.truncate(start);
			continue;
		}

		last_start = last_start.max(start);

		// Pad to 256-byte alignment
		// TODO: Why 256? Can I double this to 512 since a dual-quat is only half
		//       the size of a 4x4 matrix?
		while buffer.len() % 8 != 0 {
			buffer.push(DualQuat::ZERO);
		}

		r_skin_indices.insert(entity, DqSkinIndex::new(start));
	}

	// Pad out the buffer to ensure that there's enough space for bindings
	while r_uniform.buffer.len() - last_start < bevy::pbr::MAX_JOINTS {
		r_uniform.buffer.push(DualQuat::ZERO);
	}
}

pub fn no_automatic_skin_batching(
	mut cmd: Commands,
	q_untagged_skinned_meshes: Query<Entity, (With<DqSkinnedMesh>, Without<NoAutomaticBatching>)>,
) {
	for entity in q_untagged_skinned_meshes.iter() {
		cmd.entity(entity).try_insert(NoAutomaticBatching);
	}
}
