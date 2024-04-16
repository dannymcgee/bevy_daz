use bevy::{
	core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
	math::vec3,
	pbr::PointLightShadowMap,
	prelude::*,
	render::{
		camera::Exposure,
		mesh::{Indices, VertexAttributeValues},
	},
	utils::smallvec::{smallvec, SmallVec},
};
use bevy_daz::{DazAsset, DazAssetSourcePlugin, DazBone, DazFigure, DazPlugins};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
	let mut app = App::new();
	app.add_plugins((
		// This needs to be loaded before `DefaultPlugins`
		DazAssetSourcePlugin::with_root_paths(vec![
			"C:/Users/Public/Documents/My DAZ 3D Library".into()
		]),
		DefaultPlugins,
		// This provides the asset loader, spawning behavior, and some runtime
		// functionality like keeping child skeletons in sync with their parents
		DazPlugins,
		TemporalAntiAliasPlugin,
		DebugVisualiztionsPlugin,
	));

	app.insert_resource(PointLightShadowMap { size: 2048 })
		.insert_resource(Msaa::Off);

	app.add_systems(Startup, (spawn_environment, spawn_genesis9_figure));

	app.run();
}

// The meat of the example
fn spawn_genesis9_figure(mut cmd: Commands, r_assets: Res<AssetServer>) {
	const G9_DIR: &str = "daz://data/Daz 3D/Genesis 9";

	cmd.spawn((
		Name::new("Figure"),
		DazFigure,
		SpatialBundle::default(),
		r_assets.load::<DazAsset>(format!("{G9_DIR}/Base/Genesis9.dsf")),
	))
	.with_children(|builder| {
		builder.spawn((
			Name::new("Eyes"),
			SpatialBundle::default(),
			r_assets.load::<DazAsset>(format!("{G9_DIR}/Genesis 9 Eyes/Genesis9Eyes.dsf")),
		));
		builder.spawn((
			Name::new("Eyelashes"),
			SpatialBundle::default(),
			r_assets.load::<DazAsset>(format!(
				"{G9_DIR}/Genesis 9 Eyelashes/Genesis9Eyelashes.dsf"
			)),
		));
		builder.spawn((
			Name::new("Tear"),
			SpatialBundle::default(),
			r_assets.load::<DazAsset>(format!("{G9_DIR}/Genesis 9 Tear/Genesis9Tear.dsf")),
		));
		builder.spawn((
			Name::new("Mouth"),
			SpatialBundle::default(),
			r_assets.load::<DazAsset>(format!("{G9_DIR}/Genesis 9 Mouth/Genesis9Mouth.dsf")),
		));
	});
}

// The base environment
fn spawn_environment(
	mut cmd: Commands,
	mut ra_meshes: ResMut<Assets<Mesh>>,
	mut ra_mats: ResMut<Assets<StandardMaterial>>,
) {
	// Plane
	cmd.spawn((Name::new("Ground Plane"), MaterialMeshBundle {
		mesh: ra_meshes.add(Plane3d::new(Vec3::Y)),
		material: ra_mats.add(Color::hex("CCCCCC").unwrap()),
		transform: Transform::from_scale(Vec3::splat(5.)),
		..default()
	}));

	// Camera
	let camera_focus = vec3(0., 1.3, 0.);
	cmd.spawn((
		Name::new("Main Camera"),
		Camera3dBundle {
			transform: Transform::from_xyz(0., 1.5169, 1.3879).looking_at(camera_focus, Vec3::Y),
			exposure: Exposure { ev100: 10. },
			..default()
		},
		PanOrbitCamera {
			focus: camera_focus,
			..default()
		},
		TemporalAntiAliasBundle::default(),
	));

	// Key Light
	cmd.spawn((Name::new("Key Light"), PointLightBundle {
		point_light: PointLight {
			color: Color::WHITE,
			intensity: 15_000.,
			radius: 0.25,
			shadows_enabled: true,
			shadow_depth_bias: 0.005,
			..default()
		},
		transform: Transform::from_xyz(-0.344645, 1.67719, 0.446905),
		..default()
	}));

	// Rim Light
	cmd.spawn((Name::new("Rim Light"), PointLightBundle {
		point_light: PointLight {
			color: Color::WHITE,
			intensity: 20_000.,
			radius: 0.4,
			range: 40.,
			shadows_enabled: false,
			..default()
		},
		transform: Transform::from_xyz(-0.557236, 1.19414, -0.938458),
		..default()
	}));

	// Fill Light
	cmd.spawn((Name::new("Fill Light"), PointLightBundle {
		point_light: PointLight {
			color: Color::WHITE,
			intensity: 100_000.,
			radius: 0.5,
			range: 40.,
			shadows_enabled: false,
			..default()
		},
		transform: Transform::from_xyz(3., 1.5, 0.),
		..default()
	}));
}

// Debug visualizations
struct DebugVisualiztionsPlugin;

impl Plugin for DebugVisualiztionsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((WorldInspectorPlugin::new(), PanOrbitCameraPlugin));

		app.init_gizmo_group::<WireframeGizmoGroup>()
			.init_gizmo_group::<BoneGizmoGroup>();

		app.register_type::<OverlayVisualizations>()
			.insert_resource(OverlayVisualizations::default());

		app.add_systems(Startup, (configure_gizmos, spawn_ui));
		app.add_systems(
			Update,
			(
				configure_visualizations,
				gather_mesh_data.pipe(visualize_mesh_data),
				visualize_bones,
			),
		);
	}
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
struct OverlayVisualizations {
	pub wireframe: bool,
	pub wireframe_color: Color,

	pub normals: bool,
	pub normals_color: Color,
	pub normals_length: f32,

	pub bones: bool,
	pub bone_color: Color,
	pub bone_orientations: bool,
}

impl Default for OverlayVisualizations {
	fn default() -> Self {
		Self {
			wireframe: false,
			wireframe_color: Color::BLACK.with_a(0.2),

			normals: false,
			normals_color: Color::CYAN.with_a(0.4),
			normals_length: 0.01,

			bones: false,
			bone_color: Color::BLACK.with_a(0.6),
			bone_orientations: false,
		}
	}
}

#[derive(GizmoConfigGroup, Clone, Copy, Debug, Default, Reflect)]
struct BoneGizmoGroup;

#[derive(GizmoConfigGroup, Clone, Copy, Debug, Default, Reflect)]
struct WireframeGizmoGroup;

fn configure_gizmos(mut r_configs: ResMut<GizmoConfigStore>) {
	let (bone_config, _) = r_configs.config_mut::<BoneGizmoGroup>();
	bone_config.depth_bias = -1.;

	let (wireframe_config, _) = r_configs.config_mut::<WireframeGizmoGroup>();
	wireframe_config.depth_bias = -0.001;
}

fn spawn_ui(mut cmd: Commands) {
	fn text_hint(builder: &mut ChildBuilder<'_>, text: impl Into<String>) {
		builder.spawn(TextBundle::from_section(text, TextStyle {
			color: Color::WHITE,
			font_size: 14.,
			..default()
		}));
	}

	cmd.spawn((Name::new("UI"), NodeBundle {
		style: Style {
			flex_direction: FlexDirection::Column,
			position_type: PositionType::Absolute,
			row_gap: Val::Px(16.),
			top: Val::Px(16.),
			right: Val::Px(16.),
			..default()
		},
		..default()
	}))
	.with_children(|builder| {
		builder
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Column,
					row_gap: Val::Px(4.),
					..default()
				},
				..default()
			})
			.with_children(|builder| {
				text_hint(builder, "LMB: Orbit");
				text_hint(builder, "RMB: Pan");
				text_hint(builder, "Mouse Wheel: Zoom");
			});

		builder
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Column,
					row_gap: Val::Px(4.),
					..default()
				},
				..default()
			})
			.with_children(|builder| {
				text_hint(builder, "W: Toggle wireframes");
				text_hint(builder, "N: Toggle normals");
				text_hint(builder, "B: Toggle bones");
				text_hint(builder, "O: Toggle bone orientations");
			});
	});
}

fn configure_visualizations(
	ri_keyboard: Res<ButtonInput<KeyCode>>,
	mut r_config: ResMut<OverlayVisualizations>,
) {
	if ri_keyboard.just_pressed(KeyCode::KeyW) {
		r_config.wireframe = !r_config.wireframe;
	}
	if ri_keyboard.just_pressed(KeyCode::KeyN) {
		r_config.normals = !r_config.normals;
	}
	if ri_keyboard.just_pressed(KeyCode::KeyB) {
		r_config.bones = !r_config.bones;
	}
	if ri_keyboard.just_pressed(KeyCode::KeyO) {
		r_config.bone_orientations = !r_config.bone_orientations;
	}
}

struct MeshData {
	positions: Vec<Vec3>,
	normals: Vec<Vec3>,
	indices: Vec<usize>,
}

fn gather_mesh_data(
	r_config: Res<OverlayVisualizations>,
	ra_meshes: Res<Assets<Mesh>>,
	q_meshes: Query<&Handle<Mesh>>,
) -> Option<Vec<MeshData>> {
	if !r_config.wireframe && !r_config.normals {
		return None;
	}

	let mesh_data = q_meshes
		.iter()
		.filter_map(|mesh_handle| {
			let mesh = ra_meshes.get(mesh_handle)?;

			let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;
			let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)?;
			let indices = mesh.indices()?;

			use VertexAttributeValues::*;
			let Float32x3(positions) = positions else {
				return None;
			};
			let Float32x3(normals) = normals else {
				return None;
			};
			let Indices::U32(indices) = indices else {
				return None;
			};

			Some(MeshData {
				positions: positions.iter().copied().map(Vec3::from).collect(),
				normals: normals.iter().copied().map(Vec3::from).collect(),
				indices: indices.iter().copied().map(|idx| idx as usize).collect(),
			})
		})
		.collect();

	Some(mesh_data)
}

fn visualize_mesh_data(
	In(data): In<Option<Vec<MeshData>>>,
	mut gizmos: Gizmos<WireframeGizmoGroup>,
	r_config: Res<OverlayVisualizations>,
) {
	let Some(data) = data else {
		return;
	};
	if r_config.wireframe {
		visualize_wireframe(&data, &mut gizmos, &r_config);
	}
	if r_config.normals {
		visualize_normals(&data, &mut gizmos, &r_config);
	}
}

fn visualize_wireframe(
	data: &[MeshData],
	gizmos: &mut Gizmos<WireframeGizmoGroup>,
	config: &OverlayVisualizations,
) {
	for MeshData {
		positions, indices, ..
	} in data
	{
		for face in indices.chunks_exact(6) {
			let &[i0, i1, i2, _, _, i3] = face else {
				continue;
			};
			let v0 = positions[i0];
			let v1 = positions[i1];
			let v2 = positions[i2];
			let v3 = positions[i3];

			gizmos.line(v0, v1, config.wireframe_color);
			gizmos.line(v1, v2, config.wireframe_color);
			gizmos.line(v2, v3, config.wireframe_color);
			gizmos.line(v3, v0, config.wireframe_color);
		}
	}
}

fn visualize_normals(
	data: &[MeshData],
	gizmos: &mut Gizmos<WireframeGizmoGroup>,
	config: &OverlayVisualizations,
) {
	for MeshData {
		positions,
		normals,
		indices,
	} in data
	{
		for face in indices.chunks_exact(6) {
			let &[i0, i1, i2, _, _, i3] = face else {
				continue;
			};
			let v0 = positions[i0];
			let v1 = positions[i1];
			let v2 = positions[i2];
			let v3 = positions[i3];

			let n0 = normals[i0];
			let n1 = normals[i1];
			let n2 = normals[i2];
			let n3 = normals[i3];

			gizmos.line(v0, v0 + n0 * config.normals_length, config.normals_color);
			gizmos.line(v1, v1 + n1 * config.normals_length, config.normals_color);
			gizmos.line(v2, v2 + n2 * config.normals_length, config.normals_color);
			gizmos.line(v3, v3 + n3 * config.normals_length, config.normals_color);
		}
	}
}

fn visualize_bones(
	mut gizmos: Gizmos<BoneGizmoGroup>,
	r_config: Res<OverlayVisualizations>,
	q_bones: Query<(&GlobalTransform, &DazBone)>,
) {
	if !r_config.bones && !r_config.bone_orientations {
		return;
	}

	for (world_xform, DazBone { end_point, .. }) in q_bones.iter() {
		let head = world_xform.translation();
		let tail = world_xform.transform_point(*end_point);
		let bone_len = Vec3::distance(head, tail);
		let rot = Quat::from_affine3(&world_xform.affine());

		if r_config.bones {
			// "Head" sphere
			gizmos.sphere(head, rot, bone_len / 20., r_config.bone_color);
			// "Tail" sphere
			gizmos.sphere(tail, rot, bone_len / 20., r_config.bone_color);

			// Blender-style octahedron
			let to_tail = (tail - head).normalize_or_zero();
			if !nearly_zero(to_tail) {
				// The Daz bone orientations aren't consistent wrt which of the
				// transform's axes points toward the "tail". To draw the edge loop
				// around the body of the octahedron, we need to do the following
				// for each bone:
				//
				// 1. Figure out which of the bone transform's basis vectors points
				//    toward the tail (we can sort by `basis_vector dot to_tail`;
				//    the smallest value is the one that's roughly parallel)
				// 2. Create a 3x3 rotation matrix where the Y axis is the one
				//    pointing toward the tail, and the X and Z are any of the other
				//    two basis vectors of the original matrix
				// 3. Create a set of vertex positions for each point on the
				//    octahedron's edge loop, as if the "head" were at (0,0,0) and
				//    the "tail" were at (0,1,0)
				// 4. Use the rotation matrix and head position in world-space to
				//    move those four points into the correct world-space positions
				let mut basis_vectors: SmallVec<[Vec3; 3]> = smallvec![
					world_xform.affine().x_axis.into(),
					world_xform.affine().y_axis.into(),
					world_xform.affine().z_axis.into(),
				];

				basis_vectors.sort_unstable_by(|a, b| {
					a.dot(to_tail)
						.abs()
						.partial_cmp(&b.dot(to_tail).abs())
						.unwrap()
				});

				let rotator = Mat3::from_cols(basis_vectors[0], to_tail, basis_vectors[1]);

				let fl = vec3(-1., 1., -1.) * (bone_len / 10.);
				let fr = vec3(1., 1., -1.) * (bone_len / 10.);
				let br = vec3(1., 1., 1.) * (bone_len / 10.);
				let bl = vec3(-1., 1., 1.) * (bone_len / 10.);

				let fl = (rotator * fl) + head;
				let fr = (rotator * fr) + head;
				let br = (rotator * br) + head;
				let bl = (rotator * bl) + head;

				gizmos.line(head, fl, r_config.bone_color);
				gizmos.line(head, fr, r_config.bone_color);
				gizmos.line(head, br, r_config.bone_color);
				gizmos.line(head, bl, r_config.bone_color);

				gizmos.line(tail, fl, r_config.bone_color);
				gizmos.line(tail, fr, r_config.bone_color);
				gizmos.line(tail, br, r_config.bone_color);
				gizmos.line(tail, bl, r_config.bone_color);

				gizmos.line(fl, fr, r_config.bone_color);
				gizmos.line(fr, br, r_config.bone_color);
				gizmos.line(br, bl, r_config.bone_color);
				gizmos.line(bl, fl, r_config.bone_color);
			}
		}

		if r_config.bone_orientations {
			gizmos.line(
				head,
				head + world_xform.right() * bone_len * 0.25,
				Color::RED,
			);
			gizmos.line(
				head,
				head + world_xform.up() * bone_len * 0.25,
				Color::GREEN,
			);
			gizmos.line(
				head,
				head + world_xform.back() * bone_len * 0.25,
				Color::BLUE,
			);
		}
	}
}

fn nearly_zero(vector: Vec3) -> bool {
	let Vec3 { x, y, z } = vector;
	x.abs() <= f32::EPSILON && y.abs() <= f32::EPSILON && z.abs() <= f32::EPSILON
}
