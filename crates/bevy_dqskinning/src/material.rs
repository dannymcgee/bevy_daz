use bevy::{
	asset::Asset,
	pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial},
	reflect::Reflect,
	render::render_resource::{AsBindGroup, ShaderRef},
};

pub type DqsStandardMaterial = ExtendedMaterial<StandardMaterial, DqsMaterialExt>;

#[derive(Asset, AsBindGroup, Reflect, Clone, Debug, Default)]
pub struct DqsMaterialExt {}

impl MaterialExtension for DqsMaterialExt {
	fn prepass_vertex_shader() -> ShaderRef {
		"shaders/dqs_prepass.wgsl".into()
	}

	fn deferred_vertex_shader() -> ShaderRef {
		"shaders/dqs_vertex.wgsl".into()
	}

	fn vertex_shader() -> ShaderRef {
		"shaders/dqs_vertex.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		ShaderRef::Default
	}

	fn deferred_fragment_shader() -> ShaderRef {
		ShaderRef::Default
	}
}
