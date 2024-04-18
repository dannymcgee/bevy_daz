mod dual_quat;
mod material;

use bevy::{
	app::{App, Plugin},
	asset::{load_internal_asset, Handle},
	pbr::MaterialPlugin,
	render::render_resource::Shader,
};

pub use crate::{
	dual_quat::DualQuat,
	material::{DqsMaterialExt, DqsStandardMaterial},
};

pub const DQ_MATH_HANDLE: Handle<Shader> = Handle::weak_from_u128(13324415035412822000);
pub const DQ_SKINNING_HANDLE: Handle<Shader> = Handle::weak_from_u128(7187723715191461000);

pub struct DqSkinningPlugin;

impl Plugin for DqSkinningPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<DualQuat>();

		load_internal_asset!(app, DQ_MATH_HANDLE, "dq_math.wgsl", Shader::from_wgsl);
		load_internal_asset!(
			app,
			DQ_SKINNING_HANDLE,
			"dq_skinning.wgsl",
			Shader::from_wgsl
		);

		app.add_plugins(MaterialPlugin::<DqsStandardMaterial>::default());
	}
}
