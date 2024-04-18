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

	let dq0 = dq_math::dq_from_mat4x4(joint_xforms.data[indices.x]);
	let q0 = normalize(dq0[0]);

	var result: mat2x4<f32> = dq_math::dq_scale(dq0, weights.x);

	for (var i: u32 = 1u; i < 4; i = i + 1) {
		let k = indices[i];
		var w: f32 = weights[i];

		let m = joint_xforms.data[k];
		let dq = dq_math::dq_from_mat4x4(m);

		let rotation = normalize(dq[0]);
		if (dot(rotation, q0) < 0.0) {
			w = w * -1.0;
		}

		result = dq_math::dq_add(result, dq_math::dq_scale(dq, w));
	}

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
