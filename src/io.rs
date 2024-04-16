use std::{
	fmt::Write,
	path::{Path, PathBuf},
	sync::Arc,
};

use async_fs::File;
use bevy::{
	asset::io::{AssetReader, AssetReaderError, AssetSource, AssetSourceId, PathStream, Reader},
	prelude::*,
	utils::{hashbrown::HashMap, BoxedFuture},
};
use futures_lite::StreamExt;
use merge_streams::MergeStreams;

pub struct DazAssetSourcePlugin {
	pub root_paths: Vec<PathBuf>,
}

impl DazAssetSourcePlugin {
	pub fn with_root_paths(root_paths: Vec<PathBuf>) -> Self {
		Self { root_paths }
	}
}

impl Default for DazAssetSourcePlugin {
	fn default() -> Self {
		Self {
			root_paths: vec!["C:/Users/Public/Documents/My DAZ 3D Library".into()],
		}
	}
}

impl Plugin for DazAssetSourcePlugin {
	fn build(&self, app: &mut App) {
		let reader = DazAssetReader {
			root_paths: self.root_paths.clone(),
		};

		app.register_asset_source(
			AssetSourceId::Name("daz".into()),
			AssetSource::build().with_reader(move || Box::new(reader.clone())),
		);
	}
}

#[derive(Clone, Debug)]
pub struct DazAssetReader {
	pub root_paths: Vec<PathBuf>,
}

impl AssetReader for DazAssetReader {
	fn read<'a>(
		&'a self,
		path: &'a Path,
	) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
		Box::pin(async move {
			let mut errors = HashMap::<PathBuf, AssetReaderError>::default();

			for root_path in self.root_paths.iter() {
				let full_path = root_path.join(path);
				match File::open(&full_path).await {
					Ok(file) => {
						let reader: Box<Reader> = Box::new(file);
						return Ok(reader);
					}
					Err(err) => {
						errors.insert(full_path, AssetReaderError::Io(Arc::new(err)));
					}
				}
			}

			let base_message = format!(
				"Failed to read path \"{}\" from any of the configured root directories.",
				path.to_string_lossy(),
			);
			let path_messages = errors
				.iter()
				.fold(String::new(), |mut s, (full_path, error)| {
					writeln!(&mut s, "    - \"{}\": {error}", full_path.to_string_lossy()).unwrap();
					s
				});

			Err(AssetReaderError::Io(Arc::new(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("{base_message}\n{path_messages}"),
			))))
		})
	}

	fn read_meta<'a>(
		&'a self,
		path: &'a Path,
	) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
		Box::pin(async move { Err(AssetReaderError::NotFound(path.into())) })
	}

	fn read_directory<'a>(
		&'a self,
		path: &'a Path,
	) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
		Box::pin(async move {
			let mut streams: Vec<Box<PathStream>> = vec![];

			for root_path in self.root_paths.iter() {
				let full_path = root_path.join(path);
				if let Ok(entries) = async_fs::read_dir(&full_path).await {
					let root_path = root_path.clone();
					let mapped_stream = entries.filter_map(move |f| {
						f.ok().and_then(|dirent| {
							let path = dirent.path();
							let rel_path = path.strip_prefix(&root_path).ok()?;
							Some(rel_path.to_owned())
						})
					});

					streams.push(Box::new(mapped_stream));
				}
			}

			let merged: Box<PathStream> = Box::new(streams.merge());
			Ok(merged)
		})
	}

	fn is_directory<'a>(
		&'a self,
		path: &'a Path,
	) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
		Box::pin(async move {
			let results = self
				.root_paths
				.iter()
				.map(|root_path| {
					root_path
						.join(path)
						.metadata()
						.map_err(|err| {
							AssetReaderError::Io(Arc::new(std::io::Error::new(
								std::io::ErrorKind::Other,
								format!("{err}"),
							)))
						})
						.map(|meta| meta.file_type().is_dir())
				})
				.collect::<Vec<_>>();

			if results.iter().any(|result| matches!(result, Ok(true))) {
				Ok(true)
			} else if results.iter().any(|result| matches!(result, Ok(false))) {
				Ok(false)
			} else {
				results.first().cloned().unwrap_or(Ok(false))
			}
		})
	}
}
