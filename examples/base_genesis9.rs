use std::collections::VecDeque;

use bevy::{
	core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
	ecs::entity::EntityHashMap,
	math::{vec3, Vec3A},
	pbr::PointLightShadowMap,
	prelude::*,
	render::{
		camera::Exposure,
		mesh::{Indices, VertexAttributeValues},
	},
	utils::smallvec::{smallvec, SmallVec},
	window::PresentMode,
};
use bevy_daz::{
	DazAsset, DazAssetSourcePlugin, DazBone, DazFigure, DazPlugins, DqSkinnedMesh, DqsMaterialExt,
	DualQuat,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
	let mut app = App::new();
	app.add_plugins((
		// This needs to be loaded before `DefaultPlugins`
		DazAssetSourcePlugin::with_root_paths(vec![
			"C:/Users/Public/Documents/My DAZ 3D Library".into()
		]),
		DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				present_mode: PresentMode::AutoNoVsync,
				..default()
			}),
			..default()
		}),
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

		app.register_type::<FpsRealtime>()
			.register_type::<FpsMin>()
			.register_type::<FpsAvg>()
			.register_type::<FpsMax>();

		app.add_systems(Startup, (configure_gizmos, spawn_ui));
		app.add_systems(
			Update,
			(
				configure_visualizations,
				// gather_mesh_data.pipe(visualize_mesh_data),
				gather_mesh_data,
				visualize_bones,
				animate_left_arm,
				update_ui,
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
			wireframe_color: Color::WHITE.with_a(0.2),

			normals: false,
			normals_color: Color::CYAN.with_a(0.4),
			normals_length: 0.01,

			bones: false,
			bone_color: Color::WHITE.with_a(0.3),
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

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
struct FpsRealtime;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
struct FpsMin;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
struct FpsAvg;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
struct FpsMax;

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

	let perf_text_style = TextStyle {
		color: Color::GREEN,
		font_size: 14.,
		..default()
	};

	cmd.spawn((Name::new("Performance Overlay"), NodeBundle {
		style: Style {
			flex_direction: FlexDirection::Column,
			align_items: AlignItems::FlexEnd,
			row_gap: Val::Px(4.),
			position_type: PositionType::Absolute,
			right: Val::Px(16.),
			bottom: Val::Px(16.),
			padding: UiRect::axes(Val::Px(12.), Val::Px(8.)),
			..default()
		},
		background_color: Color::BLACK.with_a(0.5).into(),
		..default()
	}))
	.with_children(|builder| {
		builder.spawn((
			Name::new("FPS Realtime"),
			FpsRealtime,
			TextBundle::from_section("Realtime: 0.00 fps", perf_text_style.clone())
				.with_text_justify(JustifyText::Right),
		));

		builder
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Row,
					column_gap: Val::Px(12.),
					..default()
				},
				..default()
			})
			.with_children(|builder| {
				builder.spawn((
					Name::new("FPS Min"),
					FpsMin,
					TextBundle::from_section("Min: 0.00", perf_text_style.clone()),
				));
				builder.spawn((
					Name::new("FPS Avg"),
					FpsAvg,
					TextBundle::from_section("Avg: 0.00", perf_text_style.clone()),
				));
				builder.spawn((
					Name::new("FPS Max"),
					FpsMax,
					TextBundle::from_section("Max: 0.00", perf_text_style),
				));
			});
	});
}

fn update_ui(
	r_time: Res<Time>,
	q_fps_realtime: Query<Entity, With<FpsRealtime>>,
	q_fps_min: Query<Entity, With<FpsMin>>,
	q_fps_avg: Query<Entity, With<FpsAvg>>,
	q_fps_max: Query<Entity, With<FpsMax>>,
	mut q_text: Query<&mut Text>,
	mut l_last_1000_frames: Local<VecDeque<f32>>,
) {
	let Ok(realtime_ent) = q_fps_realtime.get_single() else {
		return;
	};
	let Ok(min_ent) = q_fps_min.get_single() else {
		return;
	};
	let Ok(avg_ent) = q_fps_avg.get_single() else {
		return;
	};
	let Ok(max_ent) = q_fps_max.get_single() else {
		return;
	};

	let fps_current = 1. / r_time.delta_seconds();
	if !fps_current.is_finite() {
		return;
	}

	q_text.get_mut(realtime_ent).unwrap().sections[0].value =
		format!("Realtime: {fps_current:.2} fps");

	l_last_1000_frames.push_back(fps_current);
	if l_last_1000_frames.len() > 1000 {
		l_last_1000_frames.pop_front();
	}

	let sum: f32 = l_last_1000_frames.iter().sum();
	let avg = sum / (l_last_1000_frames.len() as f32);
	let min = l_last_1000_frames
		.iter()
		.copied()
		.reduce(f32::min)
		.unwrap_or(0.);
	let max = l_last_1000_frames
		.iter()
		.copied()
		.reduce(f32::max)
		.unwrap_or(f32::MAX);

	q_text.get_mut(min_ent).unwrap().sections[0].value = format!("Min: {min:.2}");
	q_text.get_mut(avg_ent).unwrap().sections[0].value = format!("Avg: {avg:.2}");
	q_text.get_mut(max_ent).unwrap().sections[0].value = format!("Max: {max:.2}");
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

fn animate_left_arm(
	r_time: Res<Time>,
	q_figure: Query<&Children, With<DazFigure>>,
	q_children: Query<&Children>,
	q_names: Query<&Name>,
	mut q_xform: Query<&mut Transform>,
	mut l_left_arm: Local<Option<Entity>>,
) {
	if l_left_arm.is_none() {
		let g9 = q_figure.single().iter().copied().find(|&ent| {
			let name = q_names.get(ent);
			name.is_ok() && name.unwrap().as_str() == "Genesis9"
		});

		let Some(g9) = g9 else {
			return;
		};

		*l_left_arm = q_children.iter_descendants(g9).find(|&ent| {
			let name = q_names.get(ent);
			name.is_ok() && name.unwrap().as_str() == "l_upperarm"
		});
	}

	if let Some(left_arm) = l_left_arm.as_ref().copied() {
		use std::f32::consts;
		let mut xform = q_xform.get_mut(left_arm).unwrap();
		xform.rotation =
			Quat::from_rotation_z(consts::FRAC_PI_3 * (r_time.elapsed_seconds() * 2.).sin());
	}
}

struct MeshData {
	entity: Entity,
	positions: Vec<Vec3>,
	normals: Vec<Vec3>,
	indices: Vec<usize>,
	joint_indices: Vec<[usize; 4]>,
	joint_weights: Vec<[f32; 4]>,
}

fn gather_mesh_data(
	mut gizmos: Gizmos<WireframeGizmoGroup>,
	r_config: Res<OverlayVisualizations>,
	ra_meshes: Res<Assets<Mesh>>,
	q_meshes: Query<(Entity, &Handle<Mesh>)>,
	q_skinned_mesh: Query<&DqSkinnedMesh>,
	q_joints: Query<(&Name, &GlobalTransform, &DazBone)>,
) /*-> Option<Vec<MeshData>>*/
{
	if !r_config.wireframe && !r_config.normals {
		return;
	}

	let mesh_data = q_meshes
		.iter()
		.filter_map(|(entity, mesh_handle)| {
			let mesh = ra_meshes.get(mesh_handle)?;

			let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;
			let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)?;
			let indices = mesh.indices()?;
			let joint_indices = mesh.attribute(DqsMaterialExt::ATTRIBUTE_JOINT_INDEX)?;
			let joint_weights = mesh.attribute(DqsMaterialExt::ATTRIBUTE_JOINT_WEIGHT)?;

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
			let Uint16x4(joint_indices) = joint_indices else {
				return None;
			};
			let Float32x4(joint_weights) = joint_weights else {
				return None;
			};

			Some(MeshData {
				entity,
				positions: positions.iter().copied().map(Vec3::from).collect(),
				normals: normals.iter().copied().map(Vec3::from).collect(),
				indices: indices.iter().copied().map(|idx| idx as usize).collect(),
				joint_indices: joint_indices
					.iter()
					.copied()
					.map(|[a, b, c, d]| [a as usize, b as usize, c as usize, d as usize])
					.collect(),
				joint_weights: joint_weights.clone(),
			})
		})
		.collect::<Vec<_>>();

	for data in mesh_data {
		let skinned_mesh = q_skinned_mesh.get(data.entity).unwrap();
		let joints = &skinned_mesh.joints;
		let dual_quats = joints
			.iter()
			.copied()
			.map(|entity| {
				let (_, world_xform, bone) = q_joints.get(entity).unwrap();
				let joint_xform = DualQuat::from(*world_xform) * bone.inverse_bindpose;

				(entity, joint_xform)
			})
			.collect::<EntityHashMap<_>>();

		for face in data.indices.chunks_exact(6) {
			let &[i0, i1, i2, _, _, i3] = face else {
				continue;
			};
			let v0 = data.positions[i0];
			let v1 = data.positions[i1];
			let v2 = data.positions[i2];
			let v3 = data.positions[i3];

			let n0 = data.normals[i0];
			let n1 = data.normals[i1];
			let n2 = data.normals[i2];
			let n3 = data.normals[i3];

			let (v0, n0) = deform_vertex(
				&data,
				&skinned_mesh.joints,
				&dual_quats,
				i0,
				v0.into(),
				n0.into(),
			);
			let (v1, n1) = deform_vertex(
				&data,
				&skinned_mesh.joints,
				&dual_quats,
				i1,
				v1.into(),
				n1.into(),
			);
			let (v2, n2) = deform_vertex(
				&data,
				&skinned_mesh.joints,
				&dual_quats,
				i2,
				v2.into(),
				n2.into(),
			);
			let (v3, n3) = deform_vertex(
				&data,
				&skinned_mesh.joints,
				&dual_quats,
				i3,
				v3.into(),
				n3.into(),
			);

			if r_config.wireframe {
				gizmos.line(v0.into(), v1.into(), r_config.wireframe_color);
				gizmos.line(v1.into(), v2.into(), r_config.wireframe_color);
				gizmos.line(v2.into(), v3.into(), r_config.wireframe_color);
				gizmos.line(v3.into(), v0.into(), r_config.wireframe_color);
			}

			if r_config.normals {
				gizmos.line(
					v0.into(),
					(v0 + n0 * r_config.normals_length).into(),
					r_config.normals_color,
				);
				gizmos.line(
					v1.into(),
					(v1 + n1 * r_config.normals_length).into(),
					r_config.normals_color,
				);
				gizmos.line(
					v2.into(),
					(v2 + n2 * r_config.normals_length).into(),
					r_config.normals_color,
				);
				gizmos.line(
					v3.into(),
					(v3 + n3 * r_config.normals_length).into(),
					r_config.normals_color,
				);
			}
		}
	}

	// Some(mesh_data)
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

fn deform_vertex(
	data: &MeshData,
	joints: &[Entity],
	dual_quats: &EntityHashMap<DualQuat>,
	vert_idx: usize,
	pos: Vec3A,
	norm: Vec3A,
) -> (Vec3A, Vec3A) {
	let vert_joint_indices = data.joint_indices[vert_idx];
	let vert_joint_weights = data.joint_weights[vert_idx];

	let weights = vert_joint_indices
		.iter()
		.enumerate()
		.map(|(buffer_idx, joint_idx)| {
			let weight = vert_joint_weights[buffer_idx];
			let joint_ent = joints[*joint_idx];
			(joint_ent, weight)
		});

	let xform_sum = weights.fold(None, |accum: Option<DualQuat>, (joint, weight)| {
		if weight.abs() <= 1.0e-3 {
			return accum;
		}

		let xform = dual_quats[&joint];
		if let Some(acc_xform) = accum {
			Some(acc_xform + xform * weight)
		} else {
			Some(xform * weight)
		}
	});

	let xform_sum = xform_sum.unwrap_or_default();

	if xform_sum.magnitude() <= 1.0e-3 {
		return (pos, norm);
	}

	let joint_xform = xform_sum.normalize();

	(
		joint_xform.transform_point3a(pos),
		joint_xform.transform_vector3a(norm),
	)
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
		..
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

				// The shoulder bones' orientations aren't actually aligned with
				// their head -> tail vectors, which introduces some skew. This
				// honestly isn't a big deal, but I've already gone this far, so
				// let's orthogonalize the other two basis vectors for good measure.
				let basis_x = basis_vectors[0].cross(to_tail);
				let basis_z = basis_x.cross(to_tail);
				let rotator = Mat3::from_cols(basis_x, to_tail, basis_z);

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
