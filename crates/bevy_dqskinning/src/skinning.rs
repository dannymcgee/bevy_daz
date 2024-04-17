use std::ops::Deref;

use bevy::{
	asset::{Asset, Handle},
	ecs::{
		component::Component,
		entity::{Entity, EntityMapper, MapEntities},
		reflect::{ReflectComponent, ReflectMapEntities},
	},
	reflect::{Reflect, TypePath},
};

use crate::DualQuat;

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
