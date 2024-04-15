use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Describes the addressing, version, and ownership of an asset.
///
/// ## Details
///
/// The id indicates the file path that should be used for the file containing
/// this definition. The file path should always begin with a leading ‘/’ and is
/// understood to be relative to the content directory root folder.
///
/// Assets within a DSF file are assumed to “live” together forever so that
/// asset addressing of assets within the file may remain constant. If it is
/// necessary to update the assets within a previously deployed file and re-
/// eploy it, the revision number should be incremented to allow differentiation
/// between the original file and the updated file. How the revision field is
/// interpreted is application-defined.
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/asset_info/start)
#[derive(Clone, Debug, Deserialize)]
pub struct AssetInfo {
	/// A string representing the URL for this file, relative to a content root
	/// folder.
	pub id: String,
	/// A string representing a hint of how to load the file.
	pub r#type: Option<String>,
	/// A [Contributor] object representing a person or entity that worked on the
	/// asset.
	pub contributor: Contributor,
	/// A string representing the revision number for the file.
	#[serde(default = "revision_default")]
	pub revision: String,
	/// A [DateTime] representing the given revision of the file.
	pub modified: Option<DateTime<Utc>>,
}

fn revision_default() -> String {
	"1.0".into()
}

/// Information about an individual contributor.
///
/// ## Details
///
/// This is an optional object that represents original author identification
/// and contact info.
///
/// * [Reference](http://docs.daz3d.com/doku.php/public/dson_spec/object_definitions/contributor/start)
#[derive(Clone, Debug, Deserialize)]
pub struct Contributor {
	/// A string representing the name of the contributor.
	pub author: String,
	/// A string representing the email address of the contributor.
	pub email: Option<String>,
	/// A string representing the contributor's web site, if any.
	pub website: Option<String>,
}
