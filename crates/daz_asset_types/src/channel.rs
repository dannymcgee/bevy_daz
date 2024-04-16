use serde::Deserialize;

use super::util::strenum;

strenum! { ChannelType
	Float = "float",
	Alias = "alias",
	Bool = "bool",
	Color = "color",
	Enum = "enum",
	Image = "image",
	Int = "int",
	String = "string",
}

/// Defines properties of a floating-point value channel.
///
/// http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/channel_float/start
#[derive(Deserialize, Debug, Clone)]
pub struct ChannelFloat {
	// Common Channel fields ----------------------------------------------------
	/// A string representing a unique ID within the property scope of the
	/// containing object.
	pub id: String,

	/// A string representing the data type of the channel. Valid values are
	/// “alias”, “bool”, “color”, “enum”, “float”, “image”, “int” and “string”.
	/// See [Extended By](
	/// http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/channel/start#extended_by)
	pub r#type: ChannelType,

	/// A string representing the internal name to apply to this channel. An
	/// empty string is not a valid name.
	pub name: String,

	/// A string representing a user-facing label to apply to this channel.
	pub label: Option<String>,

	/// A boolean value representing a UI hint, indicating whether or not the
	/// parameter should be shown.
	#[serde(default = "visible_default")]
	pub visible: bool,

	/// A boolean value representing whether or not the parameter is allowed to
	/// be changed.
	#[serde(default)]
	pub locked: bool,

	/// A boolean value representing whether or not the channel should
	/// automatically be connected to a corresponding channel during conforming.
	#[serde(default)]
	pub auto_follow: bool,

	// Type-specific fields -----------------------------------------------------
	/// A float value representing the default value for the parameter.
	#[serde(default)]
	pub value: f32,

	/// A float value representing the current value for the parameter.
	#[serde(default)]
	pub current_value: f32,

	/// A float value representing the minimum value for the parameter, or for
	/// each component of a vector-valued channel.
	#[serde(default)]
	pub min: f32,

	/// A float value representing the maximum value for the parameter, or for
	/// each component of a vector-valued channel.
	#[serde(default = "max_default")]
	pub max: f32,

	/// A boolean value representing whether or not min and max are enforced.
	#[serde(default)]
	pub clamped: bool,

	/// A boolean value representing whether or not the parameter value should be
	/// shown to the user as a percentage.
	#[serde(default)]
	pub display_as_percent: bool,

	/// A float value representing the step size, or paging size, to use for this
	/// parameter when presenting UI to the user. Effectively a scaling value.
	/// Value applies to all components of a vector-valued channel.
	#[serde(default = "step_size_default")]
	pub step_size: f32,

	/// A boolean value representing whether or not the channel is mappable.
	#[serde(default)]
	pub mappable: bool,
}

fn visible_default() -> bool {
	true
}

impl ChannelFloat {
	pub fn new(id: impl Into<String>, name: impl Into<String>, value: f32) -> Self {
		Self {
			id: id.into(),
			r#type: ChannelType::Float,
			name: name.into(),
			label: None,
			visible: true,
			locked: false,
			auto_follow: false,
			value,
			current_value: value,
			min: 0.,
			max: 1.,
			clamped: false,
			display_as_percent: false,
			step_size: 1.,
			mappable: false,
		}
	}

	pub fn with_label(mut self, label: impl Into<String>) -> Self {
		self.label = Some(label.into());
		self
	}

	pub fn hidden(mut self) -> Self {
		self.visible = false;
		self
	}

	pub fn locked(mut self) -> Self {
		self.locked = true;
		self
	}

	pub fn auto_follow(mut self) -> Self {
		self.auto_follow = true;
		self
	}

	pub fn with_min(mut self, min: f32) -> Self {
		self.min = min;
		self
	}

	pub fn with_max(mut self, max: f32) -> Self {
		self.max = max;
		self
	}

	pub fn with_step_size(mut self, step_size: f32) -> Self {
		self.step_size = step_size;
		self
	}
}

fn max_default() -> f32 {
	1.
}

fn step_size_default() -> f32 {
	1.
}

impl From<&ChannelFloat> for f32 {
	fn from(value: &ChannelFloat) -> Self {
		value.value
	}
}

#[cfg(any(feature = "bevy", feature = "glam"))]
pub trait ChannelsAsVec3 {
	#[cfg(feature = "bevy")]
	fn as_vec3(&self) -> bevy_math::Vec3;

	#[cfg(all(feature = "glam", not(feature = "bevy")))]
	fn as_vec3(&self) -> glam::Vec3;
}

#[cfg(any(feature = "bevy", feature = "glam"))]
impl ChannelsAsVec3 for [ChannelFloat; 3] {
	#[cfg(feature = "bevy")]
	fn as_vec3(&self) -> bevy_math::Vec3 {
		bevy_math::vec3(self[0].value, self[1].value, self[2].value)
	}

	#[cfg(all(feature = "glam", not(feature = "bevy")))]
	fn as_vec3(&self) -> glam::Vec3 {
		glam::vec3(self[0].value, self[1].value, self[2].value)
	}
}
