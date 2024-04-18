use bevy::{
	asset::Asset,
	pbr::{
		ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
		StandardMaterial,
	},
	reflect::Reflect,
	render::{
		mesh::{Mesh, MeshVertexAttribute, MeshVertexBufferLayout},
		render_resource::{
			AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
			VertexFormat,
		},
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

	fn specialize(
		_pipeline: &MaterialExtensionPipeline,
		descriptor: &mut RenderPipelineDescriptor,
		layout: &MeshVertexBufferLayout,
		_key: MaterialExtensionKey<Self>,
	) -> Result<(), SpecializedMeshPipelineError> {
		let vertex_layout = layout.get_layout(&[
			Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
			Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
			Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
			Self::ATTRIBUTE_JOINT_INDEX.at_shader_location(6),
			Self::ATTRIBUTE_JOINT_WEIGHT.at_shader_location(7),
		])?;
		descriptor.vertex.buffers = vec![vertex_layout];

		Ok(())
	}
}
