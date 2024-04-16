# Daz 3D asset support for Bevy Engine

## Summary

This project comes in two parts:

* `daz_asset_types`, a standalone crate providing faifhful Rust representations
  of the [Daz 3D asset file formats](http://docs.daz3d.com/doku.php/public/dson_spec/start)
* `daz_bevy`, a crate providing plugins to load and spawn Daz 3D assets into a
  Bevy Engine application

This project is a very early work-in-progress and not yet available on
crates.io. If you'd like to see it come to fruition faster, pull requests are
welcome! (TODO: Write a contributors' guide.)

## Why?

Unreal Engine has MetaHuman, and Unity has the Digial Human package, but as it
currently stands, if you want high-fidelity character models in Bevy Engine, you
essentially need a team of highly skilled artists and engineers to create them
from scratch.

It's my opinion that Daz 3D's impressive core technology, huge marketplace
ecosystem, and reasonable prices and licensing terms make it a promising
candidate to fill that gap. Unfortunately, Daz Studio's export capabilities
leave a little something to be desired &mdash; while Daz offers reasonably
well-supported "bridge" plugins for Unreal Engine and Blender, I don't know of
any direct pipelines available from Daz to glTF, and even if they existed, there
would still remain quite a bit of work to get all the bells and whistles working
correctly in Bevy Engine.

## Tentative Road Map for `bevy_daz`
- [x] Get the base Genesis 9 figure loading in Bevy Engine with UVs, joints, and
  skin weights
- [ ] Implement dual-quaternion skinning for more faithful mesh deformations
- [ ] Translate IRAY material definitions to Bevy-compatible equivalents
- [ ] Implement corrective blend shapes and flexions for 1:1 mesh deformations
- [ ] Support "shaping" morphs for custom characters designed in Daz Studio
- [ ] 1:1 support for importing saved Daz Studio scenes as Bevy Engine scenes

## Feature Checklist for `daz_asset_types`
- [x] `asset_info`
- [ ] `bulge_binding`
- [ ] `camera`
  - [ ] `camera_orthographic`
  - [ ] `camera_perspective`
- [ ] `channel`
  - [ ] `channel_alias`
  - [ ] `channel_animation`
  - [ ] `channel_bool`
  - [ ] `channel_color`
  - [ ] `channel_enum`
  - [x] `channel_float`
  - [ ] `channel_image`
  - [ ] `channel_int`
  - [ ] `channel_string`
- [x] `contributor`
- [ ] `DAZ`
- [ ] `formula`
- [x] `geometry`
- [ ] `geometry_instance`
- [ ] `graft`
- [ ] `image`
- [ ] `image_map`
- [ ] `light`
  - [ ] `light_directional`
  - [ ] `light_point`
  - [ ] `light_spot`
- [ ] `material`
- [ ] `material_channel`
- [ ] `material_instance`
- [x] `modifier`
- [ ] `modifier_instance`
- [ ] `morph`
- [ ] `named_string_map`
- [x] `node`
- [ ] `node_instance`
- [ ] `operation`
- [ ] `oriented_box`
- [x] `polygon`
- [ ] `presentation`
- [ ] `preview`
- [ ] `region`
- [ ] `rigidity`
- [ ] `rigidity_group`
- [ ] `scene`
- [x] `skin_binding`
- [x] `uv_set`
- [ ] `uv_set_instance`
- [x] `weighted_joint`
