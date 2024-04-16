use std::fmt;

use super::{util::strenum, Array};
use serde::{
	de::{self, Visitor},
	Deserialize,
};
use serde_json as json;

/// This is an asset that defines a polygon or subdivision mesh, including the
/// region map, face grouping, material grouping, and reference to a default set
/// of UV’s.
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/geometry/start)
#[derive(Clone, Debug, Deserialize)]
pub struct Geometry {
	/// A string representing the unique ID for this asset within current file
	/// scope.
	pub id: String,

	/// A string representing the internal name for the geometry.
	pub name: Option<String>,

	/// A string representing the user-readable label for the geometry.
	pub label: Option<String>,

	/// A string representing the type of mesh represented by the vertex and
	/// facet data. Must be either “polygon_mesh” or “subdivision_surface”.
	#[serde(rename(deserialize = "type"))]
	pub r#type: Option<GeometryType>,

	/// A string representing the URI of any geometry asset that this asset was
	/// derived from.
	pub source: Option<String>,

	/// A string representing the type of edge interpolation to perform during
	/// subdivision. Must be one of “no_interpolation”, “edges_and_corners”, or
	/// “edges_only”. This is only valid when type is “subdivision_surface”.
	pub edge_interpolation_mode: Option<EdgeInterpolationMode>,

	/// A 3D-vector [Array] representing vertex positions of this geometry.
	#[cfg(feature = "bevy")]
	pub vertices: Array<bevy_math::Vec3>,
	/// A 3D-vector [Array] representing vertex positions of this geometry.
	#[cfg(all(feature = "glam", not(feature = "bevy")))]
	pub vertices: Array<glam::Vec3>,
	/// A 3D-vector [Array] representing vertex positions of this geometry.
	#[cfg(not(any(feature = "glam", feature = "bevy")))]
	pub vertices: Array<[f32; 3]>,

	/// A string [Array] representing the face group names for this geometry.
	/// Each name in the list must be unique within the list.
	pub polygon_groups: Array<String>,

	/// A string [Array] representing the material group names for this geometry.
	/// Each name in the list must be unique within the list.
	pub polygon_material_groups: Array<String>,

	/// A counted [Array] of [Polygon] objects. Polygons may not contain holes.
	pub polylist: Array<Polygon>,

	/// A string representing the URI of a default UV set for this geometry.
	pub default_uv_set: Option<String>,

	/// A [Region] object representing the root region in the region hierarchy.
	pub root_region: Option<json::Value>, // TODO

	/// A graft object representing geometry grafting information, if this object
	/// is intended to graft.
	pub graft: Option<json::Value>, // TODO

	/// A [Rigidity] object representing the rigidity map that controls how
	/// vertex weight maps should be projected onto this geometry.
	pub rigidity: Option<json::Value>, // TODO

	/// An array of objects that represent additional application-specific
	/// information for this object.
	pub extra: Option<Vec<json::Value>>,
}

strenum! { GeometryType
	PolygonMesh = "polygon_mesh",
	SubdivisionSurface = "subdivision_surface",
}

strenum! { EdgeInterpolationMode
	NoInterpolation = "no_interpolation",
	EdgesAndCorners = "edges_and_corners",
	EdgesOnly = "edges_only",
}

/// Defines an indexed polygon face.
///
/// ## Details
///
/// A polygon can define no less than three, and no more than four, vertex
/// indices. Any geometries that contain polygons with more than four vertices
/// should be broken into triangles or quads for transport in DSON format.
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/polygon/start)
#[derive(Clone, Copy, Debug, Default)]
pub struct Polygon {
	pub groups_index: usize,
	pub material_groups_index: usize,
	pub vertex_indices: (u32, u32, u32, Option<u32>),
}

struct PolygonVisitor;

impl<'a> Visitor<'a> for PolygonVisitor {
	type Value = Polygon;

	fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "an array of 5 to 6 integers")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Polygon, A::Error>
	where A: de::SeqAccess<'a> {
		let groups_index: usize = seq.next_element()?.unwrap();
		let material_groups_index: usize = seq.next_element()?.unwrap();

		let v0: u32 = seq.next_element()?.unwrap();
		let v1: u32 = seq.next_element()?.unwrap();
		let v2: u32 = seq.next_element()?.unwrap();
		let v3: Option<u32> = seq.next_element()?;

		Ok(Polygon {
			groups_index,
			material_groups_index,
			vertex_indices: (v0, v1, v2, v3),
		})
	}
}

impl<'a> Deserialize<'a> for Polygon {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where D: serde::Deserializer<'a> {
		de.deserialize_seq(PolygonVisitor)
	}
}

#[cfg(feature = "bevy")]
impl From<Geometry> for bevy_render::mesh::Mesh {
	fn from(geo: Geometry) -> Self {
		use bevy_math::Vec3;
		use bevy_render::{
			mesh::{Indices, Mesh, PrimitiveTopology},
			render_asset::RenderAssetUsages,
		};

		let positions: Vec<_> = geo.vertices.values.into_iter().map(|v| v * 0.01).collect();
		let mut normals = vec![Vec3::ZERO; positions.len()];
		let mut indices = Vec::new();

		for polygon in &geo.polylist.values {
			let (i0, i1, i2, i3) = polygon.vertex_indices;

			let v0 = positions[i0 as usize];
			let v1 = positions[i1 as usize];
			let v2 = positions[i2 as usize];

			let a = (v1 - v0).normalize();
			let b = (v2 - v0).normalize();
			let n1 = a.cross(b);

			normals[i0 as usize] += n1;
			normals[i1 as usize] += n1;
			normals[i2 as usize] += n1;

			if let Some(i3) = i3 {
				let v3 = positions[i3 as usize];
				let c = (v3 - v0).normalize();
				let n2 = b.cross(c);

				normals[i0 as usize] += n2;
				normals[i2 as usize] += n2;
				normals[i3 as usize] += n2;

				indices.extend([i0, i1, i2, i0, i2, i3]);
			} else {
				indices.extend([i0, i1, i2]);
			}

			// TODO: Polygon groups
			// TODO: Material groups
		}

		for normal in normals.iter_mut() {
			*normal = normal.normalize();
		}

		let mut mesh = Mesh::new(
			PrimitiveTopology::TriangleList,
			RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
		);
		mesh.insert_indices(Indices::U32(indices));
		mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
		mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

		mesh
	}
}
