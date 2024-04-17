mod dual_quat;
mod material;

use std::ops::Deref;

use bevy::{
	app::{App, Plugin},
	asset::{Asset, AssetApp, Handle},
	ecs::{
		component::Component,
		entity::{Entity, EntityMapper, MapEntities},
		reflect::{ReflectComponent, ReflectMapEntities},
	},
	pbr::MaterialPlugin,
	reflect::{Reflect, TypePath},
};

pub use crate::{
	dual_quat::DualQuat,
	material::{DqsMaterialExt, DqsStandardMaterial},
};

pub struct DqSkinningPlugin;

impl Plugin for DqSkinningPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<DualQuat>();
		app.register_type::<DqSkinnedMesh>();

		app.init_asset::<DqsInverseBindposes>();

		app.add_plugins(MaterialPlugin::<DqsStandardMaterial>::default());
	}
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, MapEntities)]
pub struct DqSkinnedMesh {
	pub inverse_bindposes: Handle<DqsInverseBindposes>,
	pub joints: Vec<Entity>,
}

impl MapEntities for DqSkinnedMesh {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		for joint in self.joints.iter_mut() {
			*joint = entity_mapper.map_entity(*joint);
		}
	}
}

#[derive(Asset, TypePath, Debug)]
pub struct DqsInverseBindposes(pub Box<[DualQuat]>);

impl From<Vec<DualQuat>> for DqsInverseBindposes {
	fn from(value: Vec<DualQuat>) -> Self {
		Self(value.into_boxed_slice())
	}
}

impl Deref for DqsInverseBindposes {
	type Target = [DualQuat];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
