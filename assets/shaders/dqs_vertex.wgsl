#import bevy_pbr::{
	forward_io::{Vertex, VertexOutput}
	morph,
	mesh_functions,
	view_transformations,
};

#ifdef MORPH_TARGETS
fn morph_vertex(in: Vertex) -> Vertex {
	var vertex = in;
	let weight_count = morph::layer_count();

	for (var i: u32 = 0u; i < weight_count; i ++) {
		let weight = morph::weight_at(i);
		if weight == 0.0 {
			continue;
		}

		vertex.position += weight * morph::morph(vertex.index, morph::position_offset, i);
	#ifdef VERTEX_NORMALS
		vertex.normal += weight * morph::morph(vertex.index, morph::normal_offset, i);
	#endif
	#ifdef VERTEX_TANGENTS
		vertex.tangent += vec4(weight * morph::morph(vertex.index, morph::tangent_offset, i), 0.0);
	#endif
	}

	return vertex;
}
#endif

@vertex
fn vertex(in: Vertex) -> VertexOutput {
	var out: VertexOutput;

#ifdef MORPH_TARGETS
	var vertex = morph_vertex(in);
#else
	var vertex = in;
#endif

	// TODO: Skin model
	// TODO: See https://github.com/gfx-rs/naga/issues/2416
	var model = mesh_functions::get_model_matrix(in.instance_index);

#ifdef VERTEX_NORMALS
	// TODO: Skin normals
	out.world_normal = mesh_functions::mesh_normal_local_to_world(
		vertex.normal,
		// TODO: See https://github.com/gfx-rs/naga/issues/2416
		in.instance_index
	);
#endif

#ifdef VERTEX_POSITIONS
	out.world_position = mesh_functions::mesh_position_local_to_world(
		model,
		vec4<f32>(vertex.position, 1.0)
	);
	out.position = view_transformations::position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS
	out.uv = vertex.uv;
#endif

#ifdef VERTEX_UVS_B
	out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
	out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
		model,
		vertex.tangent,
		// TODO: See https://github.com/gfx-rs/naga/issues/2416
		in.instance_index
	);
#endif

#ifdef VERTEX_COLORS
	out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
	// TODO: See https://github.com/gfx-rs/naga/issues/2416
	out.instance_index = in.instance_index;
#endif

	return out;
}
