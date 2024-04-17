use bevy::{
	asset::Asset,
	pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial},
	reflect::Reflect,
	render::{
		mesh::MeshVertexAttribute,
		render_resource::{AsBindGroup, ShaderRef, VertexFormat},
	},
};

pub type DqsStandardMaterial = ExtendedMaterial<StandardMaterial, DqsMaterialExt>;

#[derive(Asset, AsBindGroup, Reflect, Clone, Debug, Default)]
pub struct DqsMaterialExt {}

impl DqsMaterialExt {
	pub const ATTRIBUTE_JOINT_INDEX: MeshVertexAttribute =
		MeshVertexAttribute::new("JointIndex", 6927386501633859000, VertexFormat::Uint16x4);

	pub const ATTRIBUTE_JOINT_WEIGHT: MeshVertexAttribute =
		MeshVertexAttribute::new("JointWeight", 17377368814116174000, VertexFormat::Float32x4);
}

impl MaterialExtension for DqsMaterialExt {
	fn vertex_shader() -> ShaderRef {
		"shaders/dqs_vertex.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		ShaderRef::Default
	}
}
