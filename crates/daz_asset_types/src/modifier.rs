use serde::Deserialize;
use serde_json as json;

use crate::Array;

/// This element defines an individual modifier asset for a morph, a skin
/// binding, a channel, or an application-defined modifier type.
///
/// ## Details
///
/// The morph and skin properties are mutually exclusive. A modifier may exist
/// without either a morph or a skin defined, in which case it may become a data
/// repository and a channel that can be used as input and output for a formula.
///
/// For any user-facing morphs, the region should be specified, so that the
/// modifier can display in the interface for regional selection. Skin bindings
/// and corrective morphs usually do not require any user presentation so do not
/// need regions defined for them.
///
/// The name attribute may be used by applications to provide another addressing
/// mechanism for nodes in the scene. In object URI’s, if “name” is used as the
/// scheme identifier, then the value of the name attribute is used to look up
/// an item rather than using the id attribute. If the name attribute is
/// missing, applications should use the id attribute in its place wherever
/// needed.
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/modifier/start)
#[derive(Clone, Debug, Deserialize)]
pub struct Modifier {
	/// A string representing the unique ID for this asset within current file
	/// scope.
	pub id: String,

	/// A string representing the “internal” name for this object. Generally
	/// unique within any sibling modifiers.
	#[serde(default)]
	pub name: String,

	/// A string representing the user-readable label for the modifier.
	pub label: Option<String>,

	/// A string representing the URI of the modifier asset that this modifier
	/// was derived from, if any.
	#[serde(default)]
	pub source: String,

	/// A string representing the URI of the parent definition. A parent must
	/// appear above a child in the file.
	pub parent: Option<String>,

	/// A presentation containing metadata used to present an asset to the user,
	/// if this asset is a user-facing asset.
	pub presentation: Option<json::Value>, // TODO

	/// A channel definition.
	pub channel: Option<json::Value>, // TODO

	/// A string representing the region that the modifier should appear in.
	pub region: Option<json::Value>, // TODO

	/// A string representing a slash-delimited (“/”) path indicating the
	/// modifier’s group for data pathing and presentation in the UI.
	#[serde(default = "group_default")]
	pub group: String,

	/// An array of formula objects owned by this modifier.
	pub formulas: Option<Vec<json::Value>>, // TODO

	/// Any morph attached to this modifier.
	pub morph: Option<json::Value>, // TODO

	/// Any skin_binding attached to this modifier
	pub skin: Option<SkinBinding>,

	/// An array of objects that represent additional application-specific
	/// information for this object.
	pub extra: Option<Vec<json::Value>>,
}

fn group_default() -> String {
	"/".into()
}

/// A skin_binding defines the offsets and weights that relate a skin (geometry)
/// to a skeleton (a collection of nodes).
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/skin_binding/start)
#[derive(Deserialize, Debug, Clone)]
pub struct SkinBinding {
	/// A string representing the URI of the root node to bind to.
	pub node: String,

	/// A string representing the URI of the geometry to bind.
	pub geometry: String,

	/// An int representing the number of vertices expected in the mesh geometry.
	#[serde(default)]
	pub vertex_count: usize,

	/// An array of weighted_joint objects defining the binding.
	pub joints: Option<Vec<WeightedJoint>>,

	/// A named_string_map that provides a one to one mapping from face groups to
	/// nodes.
	pub selection_sets: Option<Vec<json::Value>>, // TODO
}

/// Defines one of the joints in a skin binding. For now, the binding matrix for
/// each node is assumed to be the identity matrix.
///
/// ## Details
/// * At least one of `node_weights`, `scale_weights`, and/or `local_weights`
/// must be present.
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/weighted_joint/start)
#[derive(Deserialize, Debug, Clone)]
pub struct WeightedJoint {
	/// A string representing the unique identifier for this object within the
	/// file scope.
	pub id: String,

	/// A string representing the URI of the target node in the skeleton.
	pub node: String,

	/// A float_indexed_array representing the general weight map for this joint.
	pub node_weights: Option<Array<(usize, f32)>>,

	/// A float_indexed_array representing the scale weight map for this joint.
	pub scale_weights: Option<Array<(usize, f32)>>,

	/// A collection of x, y, and z float_indexed_array objects representing the
	/// local weights for the joint.
	pub local_weights: Option<Array<(usize, f32, f32, f32)>>,

	/// A collection of x, y, and z bulge_binding objects representing the bulge
	/// binding weights for the joint.
	pub bulge_weights: Option<json::Value>, // TODO
}
