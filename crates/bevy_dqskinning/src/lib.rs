mod dual_quat;
mod material;
mod skin;
mod skinning;

use bevy::{
	app::{App, Plugin, PostUpdate},
	asset::AssetApp,
	ecs::schedule::IntoSystemConfigs,
	pbr::MaterialPlugin,
	render::{ExtractSchedule, Render, RenderApp, RenderSet},
};
use skin::{DqSkinIndices, DqSkinUniform};

pub use crate::{
	dual_quat::DualQuat,
	material::{DqsMaterialExt, DqsStandardMaterial},
	skinning::{DqSkinnedMesh, DqsInverseBindposes},
};

pub struct DqSkinningPlugin;

impl Plugin for DqSkinningPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<DualQuat>();
		app.register_type::<DqSkinnedMesh>();

		app.init_asset::<DqsInverseBindposes>();

		app.add_plugins(MaterialPlugin::<DqsStandardMaterial>::default());

		app.add_systems(PostUpdate, crate::skin::no_automatic_skin_batching);

		if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
			render_app
				.init_resource::<DqSkinUniform>()
				.init_resource::<DqSkinIndices>()
				.add_systems(ExtractSchedule, crate::skin::extract_skins)
				.add_systems(
					Render,
					crate::skin::prepare_skins.in_set(RenderSet::PrepareResources),
				);
		}
	}
}
