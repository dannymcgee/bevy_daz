#[cfg(not(target_arch = "spirv"))]
use core::fmt;
use core::ops;

use bevy::{
	core::{Pod, Zeroable},
	math::{Affine3A, Mat3A, Mat4, Quat, Vec3, Vec3A},
	reflect::Reflect,
	transform::components::GlobalTransform,
};

#[derive(Clone, Copy)]
#[cfg_attr(not(target_arch = "spirv"), derive(Reflect))]
#[repr(C)]
pub struct DualQuat(pub Quat, pub Quat);

impl DualQuat {
	pub const IDENTITY: Self = Self(
		Quat::from_xyzw(0., 0., 0., 1.),
		Quat::from_xyzw(0., 0., 0., 0.),
	);

	#[inline]
	pub fn from_rotation_translation(rotation: Quat, translation: Vec3) -> Self {
		let real = rotation;
		let Vec3 { x, y, z } = translation;
		let dual = (Quat::from_xyzw(x, y, z, 0.) * real) * 0.5;

		Self(real, dual)
	}

	#[inline(always)]
	pub fn real(&self) -> Quat {
		self.0
	}

	#[inline(always)]
	pub fn dual(&self) -> Quat {
		self.1
	}

	#[inline]
	pub fn dot(self, rhs: DualQuat) -> f32 {
		self.real().dot(rhs.real())
	}

	#[inline]
	pub fn magnitude_squared(self) -> f32 {
		self.real().length_squared()
	}

	#[inline(always)]
	pub fn length_squared(self) -> f32 {
		self.magnitude_squared()
	}

	#[inline]
	pub fn magnitude(self) -> f32 {
		self.real().length()
	}

	#[inline(always)]
	pub fn length(self) -> f32 {
		self.magnitude()
	}

	#[inline]
	pub fn normalize(self) -> Self {
		let mag = self.magnitude();
		assert!(
			mag > f32::EPSILON * 2.0,
			"Attempted to normalize a DualQuat with magnitude {mag}; \
				(({:.3} [{:.3} {:.3} {:.3}]), ({:.3} [{:.3} {:.3} {:.3}]))",
			self.0.w,
			self.0.x,
			self.0.y,
			self.0.z,
			self.1.w,
			self.1.x,
			self.1.y,
			self.1.z,
		);

		Self(self.real() / mag, self.dual() / mag)
	}

	#[inline]
	pub fn conjugate(self) -> Self {
		Self(self.real().conjugate(), self.dual().conjugate())
	}

	#[inline]
	pub fn rotation(self) -> Quat {
		self.real().normalize()
	}

	#[inline]
	pub fn translation(self) -> Vec3 {
		((self.dual() * 2.0) * self.real().conjugate()).xyz()
	}

	#[inline(always)]
	pub fn transform_point3(&self, point: Vec3) -> Vec3 {
		self.transform_point3a(point.into()).into()
	}

	#[inline]
	pub fn transform_point3a(&self, point: Vec3A) -> Vec3A {
		assert!(
			(self.length() - 1.0).abs() <= 1.0e-5,
			"DualQuat must be normalized before being used as a transform! \
			Attempted to transform point with a DualQuat with magnitude {}",
			self.length()
		);
		let real = self.real();
		let dual = self.dual();

		let real_xyz = Vec3A::from(real.xyz());
		let dual_xyz = Vec3A::from(dual.xyz());

		let translated = (dual_xyz * real.w - real_xyz * dual.w + real_xyz.cross(dual_xyz)) * 2.;
		let rotated = real * point;

		rotated + translated
	}

	#[inline(always)]
	pub fn transform_vector3(&self, vector: Vec3) -> Vec3 {
		self.transform_vector3a(vector.into()).into()
	}

	#[inline]
	pub fn transform_vector3a(&self, vector: Vec3A) -> Vec3A {
		assert!(
			(self.length() - 1.0).abs() <= 1.0e-5,
			"DualQuat must be normalized before being used as a transform! \
			Attempted to rotate vector with a DualQuat with magnitude {}",
			self.length()
		);
		self.real() * vector
	}
}

impl Default for DualQuat {
	#[inline(always)]
	fn default() -> Self {
		Self::IDENTITY
	}
}

impl ops::Mul<f32> for DualQuat {
	type Output = DualQuat;

	#[inline]
	fn mul(self, rhs: f32) -> DualQuat {
		DualQuat(self.real() * rhs, self.dual() * rhs)
	}
}

impl ops::Mul<DualQuat> for f32 {
	type Output = DualQuat;

	#[inline]
	fn mul(self, rhs: DualQuat) -> DualQuat {
		rhs * self
	}
}

impl ops::Add for DualQuat {
	type Output = DualQuat;

	#[inline]
	fn add(self, rhs: DualQuat) -> DualQuat {
		DualQuat(self.real() + rhs.real(), self.dual() + rhs.dual())
	}
}

impl ops::Mul for DualQuat {
	type Output = DualQuat;

	#[inline]
	fn mul(self, rhs: DualQuat) -> DualQuat {
		DualQuat(
			self.real() * rhs.real(),
			(self.real() * rhs.dual()) + (self.dual() * rhs.real()),
		)
	}
}

impl From<DualQuat> for Affine3A {
	#[inline]
	fn from(dq: DualQuat) -> Self {
		Self::from_rotation_translation(dq.rotation(), dq.translation())
	}
}

impl From<DualQuat> for Mat4 {
	#[inline(always)]
	fn from(value: DualQuat) -> Self {
		Mat4::from(Affine3A::from(value))
	}
}

impl From<DualQuat> for GlobalTransform {
	#[inline(always)]
	fn from(value: DualQuat) -> Self {
		GlobalTransform::from(Affine3A::from(value))
	}
}

impl From<Affine3A> for DualQuat {
	#[inline]
	fn from(value: Affine3A) -> Self {
		let (_, rotation, translation) = value.to_scale_rotation_translation();
		Self::from_rotation_translation(rotation, translation)
	}
}

impl From<Mat4> for DualQuat {
	#[inline]
	fn from(value: Mat4) -> Self {
		#[rustfmt::skip]
		let [
			m11, m12, m13, _,
			m21, m22, m23, _,
			m31, m32, m33, _,
			m41, m42, m43, _,
		] = value.to_cols_array();

		#[rustfmt::skip]
		let affine = Affine3A {
			matrix3: Mat3A::from_cols_array(&[
				m11, m12, m13,
				m21, m22, m23,
				m31, m32, m33,
			]),
			translation: Vec3A::new(m41, m42, m43),
		};

		Self::from(affine)
	}
}

impl From<GlobalTransform> for DualQuat {
	#[inline(always)]
	fn from(value: GlobalTransform) -> Self {
		Self::from(value.affine())
	}
}

#[cfg(not(target_arch = "spirv"))]
impl AsRef<[f32; 8]> for DualQuat {
	#[inline]
	fn as_ref(&self) -> &[f32; 8] {
		unsafe { &*(self as *const Self as *const [f32; 8]) }
	}
}

#[cfg(not(target_arch = "spirv"))]
impl AsMut<[f32; 8]> for DualQuat {
	#[inline]
	fn as_mut(&mut self) -> &mut [f32; 8] {
		unsafe { &mut *(self as *mut Self as *mut [f32; 8]) }
	}
}

#[cfg(not(target_arch = "spirv"))]
impl fmt::Debug for DualQuat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("DualQuat")
			.field(&self.0)
			.field(&self.1)
			.finish()
	}
}

#[cfg(not(target_arch = "spirv"))]
impl fmt::Display for DualQuat {
	#[rustfmt::skip]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"(({} [{} {} {}]), ({} [{} {} {}]))",
			&self.0.w, &self.0.x, &self.0.y, &self.0.z,
			&self.1.w, &self.1.x, &self.1.y, &self.1.z,
		)
	}
}

unsafe impl Pod for DualQuat {}
unsafe impl Zeroable for DualQuat {}

#[cfg(test)]
mod tests {
	use super::DualQuat;
	use bevy::math::{vec3, Affine3A, EulerRot, Quat, Vec3, Vec3A};

	trait NearlyEq {
		fn nearly_eq(self, rhs: Self) -> bool;
	}

	impl NearlyEq for f32 {
		fn nearly_eq(self, rhs: Self) -> bool {
			(self - rhs).abs() < 1.0e-4
		}
	}

	impl NearlyEq for Vec3 {
		fn nearly_eq(self, rhs: Self) -> bool {
			self.x.nearly_eq(rhs.x) && self.y.nearly_eq(rhs.y) && self.z.nearly_eq(rhs.z)
		}
	}

	impl NearlyEq for Vec3A {
		fn nearly_eq(self, rhs: Self) -> bool {
			self.x.nearly_eq(rhs.x) && self.y.nearly_eq(rhs.y) && self.z.nearly_eq(rhs.z)
		}
	}

	impl NearlyEq for Affine3A {
		fn nearly_eq(self, rhs: Self) -> bool {
			self.x_axis.nearly_eq(rhs.x_axis)
				&& self.y_axis.nearly_eq(rhs.y_axis)
				&& self.z_axis.nearly_eq(rhs.z_axis)
				&& self.translation.nearly_eq(rhs.translation)
		}
	}

	fn assert_mats_nearly_eq(lhs: Affine3A, rhs: Affine3A) {
		if !lhs.nearly_eq(rhs) {
			let diff_m11 = lhs.x_axis.x - rhs.x_axis.x;
			let diff_m12 = lhs.x_axis.y - rhs.x_axis.y;
			let diff_m13 = lhs.x_axis.z - rhs.x_axis.z;

			let diff_m21 = lhs.y_axis.x - rhs.y_axis.x;
			let diff_m22 = lhs.y_axis.y - rhs.y_axis.y;
			let diff_m23 = lhs.y_axis.z - rhs.y_axis.z;

			let diff_m31 = lhs.z_axis.x - rhs.z_axis.x;
			let diff_m32 = lhs.z_axis.y - rhs.z_axis.y;
			let diff_m33 = lhs.z_axis.z - rhs.z_axis.z;

			let diff_m41 = lhs.translation.x - rhs.translation.x;
			let diff_m42 = lhs.translation.y - rhs.translation.y;
			let diff_m43 = lhs.translation.z - rhs.translation.z;

			let diff_x = format!("{:+.6?}", vec3(diff_m11, diff_m12, diff_m13)).replace('+', " ");
			let diff_y = format!("{:+.6?}", vec3(diff_m21, diff_m22, diff_m23)).replace('+', " ");
			let diff_z = format!("{:+.6?}", vec3(diff_m31, diff_m32, diff_m33)).replace('+', " ");
			let diff_w = format!("{:+.6?}", vec3(diff_m41, diff_m42, diff_m43)).replace('+', " ");

			eprintln!("Difference:");
			eprintln!("    {diff_x}");
			eprintln!("    {diff_y}");
			eprintln!("    {diff_z}");
			eprintln!("    {diff_w}");

			panic!();
		}
	}

	#[test]
	fn matrix_conversion() {
		let r0 = Quat::from_euler(EulerRot::XYZ, 1., 2., 3.);
		let r1 = Quat::from_euler(EulerRot::XYZ, -1., 3., 2.);
		let r2 = Quat::from_euler(EulerRot::XYZ, 2., 3., 1.5);

		let t0 = vec3(10., 30., 90.);
		let t1 = vec3(30., 40., 190.);
		let t2 = vec3(5., 20., 66.);

		let dq0 = DualQuat::from_rotation_translation(r0, t0);
		let dq1 = DualQuat::from_rotation_translation(r1, t1);
		let dq2 = DualQuat::from_rotation_translation(r2, t2);
		let dq = dq0 * dq1 * dq2;

		let dq_to_mat = Affine3A::from(dq);

		let m0 = Affine3A::from_rotation_translation(r0, t0);
		let m1 = Affine3A::from_rotation_translation(r1, t1);
		let m2 = Affine3A::from_rotation_translation(r2, t2);
		let m = m0 * m1 * m2;

		assert_mats_nearly_eq(dq_to_mat, m);
	}
}
