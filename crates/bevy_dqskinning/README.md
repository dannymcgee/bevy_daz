# Dual Quaternion Skinning for Bevy Engine

This crate provides a tiny Bevy Engine plugin implementing [dual quaternion
skinning](https://users.cs.utah.edu/~ladislav/kavan07skinning/kavan07skinning.pdf)
via a `MaterialExtension` with custom vertex shaders.

If your model is already using the Bevy `StandardMaterial`, you can upgrade it
to dual quaternion skinning by replacing the material with a
`DqsStandardMaterial`:

### Before:

```rs
use bevy::prelude::*;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, spawn_model)
		.run();
}

fn spawn_model(
	mut commands: Commands,
	assets: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut std_mats: ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn(MaterialMeshBundle {
		mesh: assets.load("path/to/my/mesh"),
		material: std_mats.add(StandardMaterial {
			base_color: Color::hex("AAAAAA").unwrap(),
			metallic: 0.,
			perceptual_roughness: 0.55,
			reflectance: 0.45,
			..default()
		}),
		..default()
	});
}
```

### After:

```rs
use bevy::prelude::*;
use bevy::dq_skinning::{DqSkinningPlugin, DqsStandardMaterial, DqsMaterialExt};

fn main() {
	App::new()
		.add_plugins((DefaultPlugins, DqSkinningPlugin))
		.add_systems(Startup, spawn_model)
		.run();
}

fn spawn_model(
	mut commands: Commands,
	assets: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut dqs_mats: ResMut<Assets<DqsStandardMaterial>>,
) {
	commands.spawn(MaterialMeshBundle {
		mesh: assets.load("path/to/my/mesh"),
		material: dqs_mats.add(ExtendedMaterial {
			base: StandardMaterial {
				base_color: Color::hex("AAAAAA").unwrap(),
				metallic: 0.,
				perceptual_roughness: 0.55,
				reflectance: 0.45,
				..default()
			},
			extension: DqsMaterialExt::default(),
		}),
		..default()
	});
}
```

If you're _not_ using the `StandardMaterial`, depending on your vertex shader,
you can either replace the `ExtendedMaterial::base` field in the example above
with your custom material, or you can update your custom vertex shader to use
the functions provided in the `bevy_dqskinning::dq_skinning` and/or
`bevy_dqskinning::dq_math` shader modules depending on your needs.
