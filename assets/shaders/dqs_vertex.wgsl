#import bevy_pbr::{
	morph,
	mesh_functions,
	view_transformations
}

struct Vertex {
	@builtin(instance_index) instance_index: u32,
	@location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
#ifdef VERTEX_UVS_B
	@location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
	@location(4) tangent: vec4<f32>,
#endif
	@location(6) joint_indices: vec4<u32>,
	@location(7) joint_weights: vec4<f32>,
#ifdef MORPH_TARGETS
	@builtin(vertex_index) index: u32,
#endif
}

struct VertexOutput {
	// This is `clip position` when the struct is used as a vertex stage output
	// and `frag coord` when used as a fragment stage input
	@builtin(position) position: vec4<f32>,
	@location(0) world_position: vec4<f32>,
	@location(1) world_normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
#ifdef VERTEX_UVS_B
	@location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
	@location(4) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
	@location(6) @interpolate(flat) instance_index: u32,
#endif
}

#ifdef DQ_SKINNED
struct DqSkinnedMesh {
	data: array<mat2x4<f32>, 256u>,
}

@group(1) @binding(1)
var<uniform> joint_xforms: DqSkinnedMesh;

/// Dual-quaternion scale
fn dq_scale(dq: mat2x4<f32>, scale: f32) -> mat2x4<f32> {
	return mat2x4<f32>(
		dq[0] * scale,
		dq[1] * scale
	);
}

/// Dual-quaternion addition
fn dq_add(lhs: mat2x4<f32>, rhs: mat2x4<f32>) -> mat2x4<f32> {
	return mat2x4<f32>(
		lhs[0] + rhs[0],
		lhs[1] + rhs[1]
	);
}

/// Dual-quaternion normalization
fn dq_normalize(dq: mat2x4<f32>) -> mat2x4<f32> {
	let mag = length(dq[0]);
	if (mag <= 0.001) {
		return mat2x4<f32>(
			vec4<f32>(0.0, 0.0, 0.0, 1.0),
			vec4<f32>(0.0, 0.0, 0.0, 0.0)
		);
	}

	return mat2x4<f32>(dq[0] / mag, dq[1] / mag);
}

/// Quaternion multiplication
fn q_mul(lhs: vec4<f32>, rhs: vec4<f32>) -> vec4<f32> {
	let w = (lhs.w * rhs.w) - dot(lhs.xyz, rhs.xyz);
	let xyz = (lhs.w * rhs.xyz) + (rhs.w * lhs.xyz) + cross(lhs.xyz, rhs.xyz);

	return vec4<f32>(xyz, w);
}

/// Dual-quaternion to 4x4 transform matrix
fn mat4x4_from_dq(dq: mat2x4<f32>) -> mat4x4<f32> {
	// Convert the "real" quaternion to a 3x3 rotation matrix
	let rotation = dq[0];

	// Mostly copy-pasted from glam::Mat3A::from_quat
	let x2 = rotation.x + rotation.x;
	let y2 = rotation.y + rotation.y;
	let z2 = rotation.z + rotation.z;
	let xx = rotation.x * x2;
	let xy = rotation.x * y2;
	let xz = rotation.x * z2;
	let yy = rotation.y * y2;
	let yz = rotation.y * z2;
	let zz = rotation.z * z2;
	let wx = rotation.w * x2;
	let wy = rotation.w * y2;
	let wz = rotation.w * z2;

	let m11_m12_m13 = vec3<f32>(1.0-(yy+zz), xy+wz, xz-wy);
	let m21_m22_m23 = vec3<f32>(xy-wz, 1.0-(xx+zz), yz+wx);
	let m31_m32_m33 = vec3<f32>(xz+wy, yz-wx, 1.0-(xx+yy));

	// Extract translation vector from the dual-quat
	let lhs = dq[1] * 2.0;
	let rhs = vec4<f32>(-dq[0].xyz, dq[0].w);
	let product = q_mul(lhs, rhs);

	let m41_m42_m43 = product.xyz;

	return mat4x4<f32>(
		vec4<f32>(m11_m12_m13, 0.0),
		vec4<f32>(m21_m22_m23, 0.0),
		vec4<f32>(m31_m32_m33, 0.0),
		vec4<f32>(m41_m42_m43, 1.0)
	);
}

fn skin_model(
	indices: vec4<u32>,
	weights: vec4<f32>,
) -> mat4x4<f32> {
	if ((weights.x + weights.y + weights.z + weights.w) <= 0.001) {
		return mat4x4<f32>(
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0
		);
	}

	var result: mat2x4<f32> = dq_scale(joint_xforms.data[indices.x], weights.x);
	result = dq_add(result, dq_scale(joint_xforms.data[indices.y], weights.y));
	result = dq_add(result, dq_scale(joint_xforms.data[indices.z], weights.z));
	result = dq_add(result, dq_scale(joint_xforms.data[indices.w], weights.w));

	return mat4x4_from_dq(dq_normalize(result));
}

fn inverse_transpose_3x3m(in: mat3x3<f32>) -> mat3x3<f32> {
	let x = cross(in[1], in[2]);
	let y = cross(in[2], in[0]);
	let z = cross(in[0], in[1]);
	let det = dot(in[2], z);
	return mat3x3<f32>(
		x / det,
		y / det,
		z / det
	);
}

fn skin_normals(
	model: mat4x4<f32>,
	normal: vec3<f32>,
) -> vec3<f32> {
	return normalize(
		inverse_transpose_3x3m(
			mat3x3<f32>(
				model[0].xyz,
				model[1].xyz,
				model[2].xyz
			)
		) * normal
	);
}
#endif // DQ_SKINNED

#ifdef MORPH_TARGETS
fn morph_vertex(in: Vertex) -> Vertex {
	var vertex = in;
	let weight_count = morph::layer_count();

	for (var i: u32 = 0u; i < weight_count; i = i + 1) {
		let weight = morph::weight_at(i);
		if (weight == 0.0) {
			continue;
		}

		vertex.position += weight * morph::morph(vertex.index, morph::position_offset, i);
		vertex.normal += weight * morph::morph(vertex.index, morph::normal_offset, i);
#ifdef VERTEX_TANGENTS
		vertex.tangent += vec4(weight * morph::morph(vertex.index, morph::tangent_offset, i), 0.0);
#endif
	}

	return vertex;
}
#endif

@vertex
fn vertex(in: Vertex) -> VertexOutput {
	var out: VertexOutput;

#ifdef MORPH_TARGETS
	var vertex = morph_vertex(in);
#else
	var vertex = in;
#endif

#ifdef DQ_SKINNED
	var model = skin_model(vertex.joint_indices, vertex.joint_weights);
	out.world_normal = skin_normals(model, vertex.normal);
#else
	// TODO: See https://github.com/gfx-rs/naga/issues/2416
	var model = mesh_functions::get_model_matrix(in.instance_index);
	out.world_normal = mesh_functions::mesh_normal_local_to_world(
		vertex.normal,
		// TODO: See https://github.com/gfx-rs/naga/issues/2416
		in.instance_index
	);
#endif

	out.world_position = mesh_functions::mesh_position_local_to_world(
		model,
		vec4<f32>(vertex.position, 1.0)
	);
	out.position = view_transformations::position_world_to_clip(out.world_position.xyz);

	out.uv = vertex.uv;

#ifdef VERTEX_UVS_B
	out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
	out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
		model,
		vertex.tangent,
		// TODO: See https://github.com/gfx-rs/naga/issues/2416
		in.instance_index
	);
#endif

#ifdef VERTEX_COLORS
	out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
	// TODO: See https://github.com/gfx-rs/naga/issues/2416
	out.instance_index = in.instance_index;
#endif

	return out;
}
