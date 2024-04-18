#define_import_path bevy_dqskinning::dq_math

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

/// 3x3 rotation matrix to quaternion
fn q_from_mat3x3(mat3: mat3x3<f32>) -> vec4<f32> {
	// Adapted from glam::Quat::from_rotation_axes
	if (mat3[2].z <= 0.0) {
		// x^2 + y^2 >= z^2 + w^2
		let dif10 = mat3[1].y - mat3[0].x;
		let omm22 = 1.0 - mat3[2].z;
		if (dif10 <= 0.0) {
			// x^2 >= y^2
			let four_xsq = omm22 - dif10;
			let inv4x = 0.5 / sqrt(four_xsq);

			return vec4<f32>(
				four_xsq * inv4x,
				(mat3[0].y + mat3[1].x) * inv4x,
				(mat3[0].z + mat3[2].x) * inv4x,
				(mat3[1].z - mat3[2].y) * inv4x
			);
		} else {
			// y^2 >= x^2
			let four_ysq = omm22 + dif10;
			let inv4y = 0.5 / sqrt(four_ysq);

			return vec4<f32>(
				(mat3[0].y + mat3[1].x) * inv4y,
				four_ysq * inv4y,
				(mat3[1].z + mat3[2].y) * inv4y,
				(mat3[2].x - mat3[0].z) * inv4y
			);
		}
	} else {
		// z^2 + w^2 >= x^2 + y^2
		let sum10 = mat3[1].y + mat3[0].x;
		let opm22 = 1.0 + mat3[2].z;
		if (sum10 <= 0.0) {
			// z^2 >= w^2
			let four_zsq = opm22 - sum10;
			let inv4z = 0.5 / sqrt(four_zsq);

			return vec4<f32>(
				(mat3[0].z + mat3[2].x) * inv4z,
				(mat3[1].z + mat3[2].y) * inv4z,
				four_zsq * inv4z,
				(mat3[0].y - mat3[1].x) * inv4z
			);
		} else {
			// w^2 >= z^2
			let four_wsq = opm22 + sum10;
			let inv4w = 0.5 / sqrt(four_wsq);

			return vec4<f32>(
				(mat3[1].z - mat3[2].y) * inv4w,
				(mat3[2].x - mat3[0].z) * inv4w,
				(mat3[0].y - mat3[1].x) * inv4w,
				four_wsq * inv4w
			);
		}
	}
}

/// 4x4 transform matrix to dual-quaternion
fn dq_from_mat4x4(mat: mat4x4<f32>) -> mat2x4<f32> {
	let mat3_scaled = mat3x3<f32>(
		mat[0].xyz,
		mat[1].xyz,
		mat[2].xyz
	);
	let det = determinant(mat3_scaled);
	if (det <= 0.001) {
		return mat2x4<f32>(
			vec4<f32>(0.0, 0.0, 0.0, 1.0),
			vec4<f32>(0.0, 0.0, 0.0, 0.0)
		);
	}

	let scale = vec3<f32>(
		length(mat3_scaled[0]) * sign(det),
		length(mat3_scaled[1]),
		length(mat3_scaled[2])
	);
	let inv_scale = 1.0 / scale;

	let mat3 = mat3x3<f32>(
		mat3_scaled[0] * inv_scale.x,
		mat3_scaled[1] * inv_scale.y,
		mat3_scaled[2] * inv_scale.z
	);

	let real = q_from_mat3x3(mat3);

	let translation = mat[3].xyz;
	let dual = q_mul(vec4<f32>(translation, 0.0), real) * 0.5;

	return mat2x4<f32>(real, dual);
}
