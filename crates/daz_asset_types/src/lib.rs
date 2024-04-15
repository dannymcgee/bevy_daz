use serde::Deserialize;
use serde_json as json;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Array<T> {
	pub count: usize,
	pub values: Vec<T>,
}

/// A DAZ object is the top level object in a DSON format file.
///
/// ### Details
/// A file must contain one or more of any of the `*_library` elements and/or a
/// `scene`.
///
/// http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/daz/start
#[derive(Clone, Debug, Deserialize)]
pub struct Daz {
	/// A string indicating the file format schema version to be used when
	/// parsing the file, in the form “major.minor.revision”.
	pub file_version: String,
	/// A base-level asset_info object to apply to all assets within the file.
	pub asset_info: json::Value, // TODO
	/// An array of geometry assets defined in this file.
	pub geometry_library: Option<Vec<json::Value>>, // TODO
	/// An array of node assets defined in this file.
	pub node_library: Option<Vec<json::Value>>, // TODO
	/// An array of uv_set assets defined in this file.
	pub uv_set_library: Option<Vec<json::Value>>, // TODO
	/// An array of modifier assets defined in this file.
	pub modifier_library: Option<Vec<json::Value>>, // TODO
	/// An array of image assets defined in this file.
	pub image_library: Option<Vec<json::Value>>, // TODO
	/// An array of material assets defined in this file.
	pub material_library: Option<Vec<json::Value>>, // TODO
	/// A scene object that instantiates and configures assets to add to a
	/// current scene.
	pub scene: Option<json::Value>, // TODO
}
