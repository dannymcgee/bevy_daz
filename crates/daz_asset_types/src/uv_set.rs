use serde::Deserialize;

use crate::Array;

/// The definition of a single UV coordinate set asset.
///
/// ## Details
/// The `uvs` array is assumed to be given in the same order as the vertices in
/// any associated geometry. Any vertices that have more than one UV coordinate
/// associated with them (e.g. along a UV boundary) should appear in the
/// `polygon_vertex_indices` array. For vertices that appear in the
/// `polygon_vertex_indices` array, their entry in the `uvs` array should
/// correspond to a “primary” value for the UV's for that vertex. The “primary”
/// value may be arbitrary for most applications.
///
/// ## Example
/// For a geometry with unique UV's at each vertex, the `uvs` array size will
/// match the vertex array size of the corresponding geometry. If the geometry
/// has vertices with more than one UV associated with them, there will be an
/// entry in the `polygon_vertex_indices` array for each additional UV
/// associated with each vertex.
///
/// ```json
/// {
///   "id" : "default",
///   "name" : "default",
///   "label" : "Default UVs",
///   "vertex_count" : 266,
///   "uvs" : [
///     [ 0.02083333, 0 ],
///     [ 0.02083333, 1 ],
///     [ 0, 0.08333334 ],
///     [ 0, 0.1666667 ],
///     [ 0, 0.25 ],
///     [ 0, 0.3333333 ],
///     [ 0, 0.4166667 ],
///     [ 0, 0.5 ],
///     [ 0, 0.5833333 ],
///     [ 0, 0.6666667 ],
///     [ 0, 0.75 ],
///     [ 0, 0.8333333 ],
///     [ 0, 0.9166667 ]
///   ],
///   "polygon_vertex_indices" : [
///     [ 12, 0, 266 ],
///     [ 24, 0, 4 ],
///     [ 36, 0, 7 ],
///     [ 48, 0, 9 ],
///     [ 60, 0, 2 ]
///   ]
/// }
/// ```
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/uv_set/start)
#[derive(Clone, Debug, Deserialize)]
pub struct UvSet {
	/// A string representing a unique ID for this asset within the current file
	/// scope.
	pub id: String,

	/// A string representing the “internal” name for this UV set. Generally
	/// unique within any sibling uv sets.
	#[serde(default)]
	pub name: Option<String>,

	/// A string representing the user facing label for this UV set.
	#[serde(default)]
	pub label: Option<String>,

	/// A string representing the URI of the uv_set asset that this uv_set was
	/// derived from, if any.
	#[serde(default)]
	pub source: Option<String>,

	/// An int representing the number of vertices expected to be in the geometry
	/// that the UV set applies to.
	pub vertex_count: usize,

	/// A 2D-vector [Array] of all UV's, both those that are shared by all
	/// polygons attached to a vertex, and those that may be unique to a
	/// particular polygon attached to a vertex.
	#[cfg(feature = "bevy")]
	pub uvs: Array<bevy_math::Vec2>,
	/// A 2D-vector [Array] of all UV's, both those that are shared by all
	/// polygons attached to a vertex, and those that may be unique to a
	/// particular polygon attached to a vertex.
	#[cfg(all(feature = "glam", not(feature = "bevy")))]
	pub uvs: Array<glam::Vec2>,
	/// A 2D-vector [Array] of all UV's, both those that are shared by all
	/// polygons attached to a vertex, and those that may be unique to a
	/// particular polygon attached to a vertex.
	#[cfg(not(any(feature = "glam", feature = "bevy")))]
	pub uvs: Array<[f32; 2]>,

	/// An array representing the polygons that use a UV index for a given vertex
	/// that is different than the default index defined in “uvs”.
	/// Polygon-specific vertex-indexed UV indices. Each entry is
	/// [`polygon_index`, `polygon_vertex_index`, `uv_index`], where
	/// `polygon_vertex_index` refers to the index of a vertex in the geometry
	/// that is used by the polygon at `polygon_index`.
	pub polygon_vertex_indices: Option<Vec<[usize; 3]>>,
}
