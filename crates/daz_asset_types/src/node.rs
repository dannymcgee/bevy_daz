use serde::Deserialize;
use serde_json as json;

use crate::channel::ChannelFloat;

use super::util::strenum;

/// This object defines a node that makes up part of a node hierarchy. It can
/// represent nodes of a variety of types such as bones and figure roots.
///
/// ## Details
///
/// If type is set to “figure” then this node is understood to be the root node
/// of a figure.
///
/// The name attribute may be used by applications to provide another addressing
/// mechanism for nodes in the scene. In object URI’s, if “name” is used as the
/// scheme identifier, then the value of the name attribute is used to look up
/// an item rather than using the id attribute. If the name attribute is
/// missing, applications should use the id attribute in its place wherever
/// needed.
///
/// The translation, rotation, scale, and general_scale elements each represent
/// transforms that convert to transform matrices. To arrive at the full base
/// transform for the node, each of those elements is converted to matrix form.
/// The full transform for a node is determined using the following algorithm:
///
/// - center_offset = center_point - parent.center_point
/// - global_translation = parent.global_transform * (center_offset + translation)
/// - global_rotation = parent.global_rotation * orientation * rotation * (orientation)-1
/// - global_scale for nodes that inherit scale = parent.global_scale * orientation * scale * general_scale * (orientation)^-1
/// - global_scale for nodes = parent.global_scale * (parent.local_scale)^-1 * orientation * scale * general_scale * (orientation)^-1
/// - global_transform = global_translation * global_rotation * global_scale
///
/// Vertices are taken to global space by post-multiplying as follows:
///
/// - global_vertex = global_transform * vertex
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/node/start)
#[derive(Clone, Debug, Deserialize)]
pub struct Node {
	/// A string representing the unique ID for this asset within the current
	/// file scope.
	pub id: String,

	/// A string representing the “internal” name for this node. Generally unique
	/// within any sibling nodes.
	pub name: String,

	/// A string representing the base type for this node. Can be “node”, “bone”,
	/// “figure”, “camera”, or “light”. See [Extended By](
	/// http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/node/start#extended_by).
	#[serde(default)]
	pub r#type: NodeType,

	/// A string representing the user facing label for this node.
	pub label: String,

	/// A string representing the URI of the node asset that this node asset was
	/// derived from.
	#[serde(default)]
	pub source: String,

	/// A string representing the URI of the parent node definition. Parents must
	/// appear above children in the file.
	pub parent: Option<String>,

	/// A string representing the rotation order to use when interpreting
	/// channel-based animation data for this node. Valid values are “XYZ”,
	/// “YZX”, “ZYX”, “ZXY”, “XZY”, and “YXZ”.
	#[serde(default)]
	pub rotation_order: RotationOrder,

	/// A boolean value indicating whether or not the immediate parent node's
	/// local scale is compensated for when calculating this node's world space
	/// transform. If false, this node's world space transform is multiplied by
	/// the inverse of parent node's local scale.
	#[serde(default = "inherits_scale_default")]
	pub inherits_scale: bool,

	/// An array of x, y, and z channel_float definitions for the center point of
	/// this node.
	#[serde(default = "center_point_default")]
	pub center_point: [ChannelFloat; 3],

	/// An array of x, y, and z channel_float definitions for the end point of
	/// this node.
	#[serde(default = "end_point_default")]
	pub end_point: [ChannelFloat; 3],

	/// An array of x, y, and z channel_float definitions for the (Euler)
	/// rotation of this node.
	#[serde(default = "orientation_default")]
	pub orientation: [ChannelFloat; 3],

	/// An array of x, y, and z channel_float definitions for the (Euler)
	/// rotation of this node.
	#[serde(default = "rotation_default")]
	pub rotation: [ChannelFloat; 3],

	/// An array of x, y, and z channel_float definitions for the translation of
	/// this node.
	#[serde(default = "translation_default")]
	pub translation: [ChannelFloat; 3],

	/// An array of x, y, and z channel_float definitions for the individual
	/// (i.e. x, y, or z-axis) scale of this node.
	#[serde(default = "scale_default")]
	pub scale: [ChannelFloat; 3],

	/// A channel_float definition for the general (i.e. 3-axis) scale of this node.
	#[serde(default = "general_scale_default")]
	pub general_scale: ChannelFloat,

	/// A presentation object representing the user-facing presentation
	/// information for this node.
	pub presentation: Option<json::Value>, // TODO

	/// An array of formula objects owned by this node.
	pub formulas: Option<Vec<json::Value>>, // TODO

	/// An array of objects that represent additional application-specific
	/// information for this object.
	pub extra: Option<Vec<json::Value>>,
}

fn inherits_scale_default() -> bool {
	true
}

fn center_point_default() -> [ChannelFloat; 3] {
	[
		ChannelFloat::new("x", "xOrigin", 0.),
		ChannelFloat::new("y", "yOrigin", 0.),
		ChannelFloat::new("z", "zOrigin", 0.),
	]
}

fn end_point_default() -> [ChannelFloat; 3] {
	[
		ChannelFloat::new("x", "xEnd", 0.),
		ChannelFloat::new("y", "yEnd", 0.),
		ChannelFloat::new("z", "zEnd", 0.),
	]
}

fn orientation_default() -> [ChannelFloat; 3] {
	[
		ChannelFloat::new("x", "xOrientation", 0.),
		ChannelFloat::new("y", "yOrientation", 0.),
		ChannelFloat::new("z", "zOrientation", 0.),
	]
}

fn rotation_default() -> [ChannelFloat; 3] {
	[
		ChannelFloat::new("x", "xRotation", 0.),
		ChannelFloat::new("y", "yRotation", 0.),
		ChannelFloat::new("z", "zRotation", 0.),
	]
}

fn translation_default() -> [ChannelFloat; 3] {
	[
		ChannelFloat::new("x", "xTranslation", 0.),
		ChannelFloat::new("y", "yTranslation", 0.),
		ChannelFloat::new("z", "zTranslation", 0.),
	]
}

fn scale_default() -> [ChannelFloat; 3] {
	[
		ChannelFloat::new("x", "xScale", 0.),
		ChannelFloat::new("y", "yScale", 0.),
		ChannelFloat::new("z", "zScale", 0.),
	]
}

fn general_scale_default() -> ChannelFloat {
	ChannelFloat::new("general_scale", "Scale", 1.)
		.with_label("Scale")
		.with_min(-10000.)
		.with_max(10000.)
		.with_step_size(0.005)
}

strenum! { NodeType
	Node = "node",
	Bone = "bone",
	Figure = "figure",
	Camera = "camera",
	Light = "light",
}

strenum! { RotationOrder
	XYZ = "XYZ",
	YZX = "YZX",
	ZYX = "ZYX",
	ZXY = "ZXY",
	XZY = "XZY",
	YXZ = "YXZ",
}
