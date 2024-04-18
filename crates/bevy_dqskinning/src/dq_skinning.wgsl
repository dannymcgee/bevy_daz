#define_import_path bevy_dqskinning::dq_skinning

#import bevy_pbr::mesh_types::SkinnedMesh
#import bevy_dqskinning::dq_math

#ifdef SKINNED

@group(1) @binding(1)
var<uniform> joint_xforms: SkinnedMesh;

fn skin_model(
	indices: vec4<u32>,
	weights: vec4<f32>
) -> mat4x4<f32> {
	if ((weights.x + weights.y + weights.z + weights.w) <= 0.001) {
		return mat4x4<f32>(
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0
		);
	}

	let m0 = joint_xforms.data[indices.x];
	let m1 = joint_xforms.data[indices.y];
	let m2 = joint_xforms.data[indices.z];
	let m3 = joint_xforms.data[indices.w];

	let dq0 = dq_math::dq_from_mat4x4(m0);
	let dq1 = dq_math::dq_from_mat4x4(m1);
	let dq2 = dq_math::dq_from_mat4x4(m2);
	let dq3 = dq_math::dq_from_mat4x4(m3);

	var result: mat2x4<f32> = dq_math::dq_scale(dq0, weights.x);
	result = dq_math::dq_add(result, dq_math::dq_scale(dq1, weights.y));
	result = dq_math::dq_add(result, dq_math::dq_scale(dq2, weights.z));
	result = dq_math::dq_add(result, dq_math::dq_scale(dq3, weights.w));

	return dq_math::mat4x4_from_dq(dq_math::dq_normalize(result));
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

#endif
