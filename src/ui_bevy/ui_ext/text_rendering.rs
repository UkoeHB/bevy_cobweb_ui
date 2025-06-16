use bevy::prelude::*;
use bevy::render::sync_world::TemporaryRenderEntity;
use bevy::render::{Extract, RenderApp};
use bevy::text::{PositionedGlyph, TextLayoutInfo};
use bevy::ui::{ExtractedGlyph, ExtractedUiItem, ExtractedUiNode, ExtractedUiNodes, RenderUiSystem, UiCameraMap};

use super::{TextOutline, TextShadowGroup};

//-------------------------------------------------------------------------------------------------------------------

fn extract_text_outlines(
    mut aa_glyph_cache: Local<Vec<ExtractedGlyph>>,
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    texture_atlases: Extract<Res<Assets<TextureAtlasLayout>>>,
    uinode_query: Extract<
        Query<(
            Entity,
            &ComputedNode,
            &ComputedNodeTarget,
            &GlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            &TextLayoutInfo,
            &TextOutline,
        )>,
    >,
    camera_map: Extract<UiCameraMap>,
)
{
    aa_glyph_cache.clear();
    let mut start = extracted_uinodes.glyphs.len();
    let mut len = 0;

    let mut camera_mapper = camera_map.get_mapper();
    for (entity, uinode, target, global_transform, inherited_visibility, clip, text_layout_info, outline) in
        &uinode_query
    {
        // Skip if not visible or if size is set to zero (e.g. when a parent is set to `Display::None`)
        if !inherited_visibility.get() || uinode.is_empty() || outline.width == 0.0 {
            continue;
        }

        let Some(extracted_camera_entity) = camera_mapper.map(target) else {
            continue;
        };

        let width = (outline.width / uinode.inverse_scale_factor()).ceil() as i32;
        let width_pow2 = width.pow(2);
        let aa_factor = outline.anti_aliasing.unwrap_or(1.0);
        let color: LinearRgba = outline.color.into();
        let mut aa_color = color;
        aa_color.alpha *= aa_factor;

        for (i, PositionedGlyph { position, atlas_info, .. }) in text_layout_info.glyphs.iter().enumerate() {
            let rect = texture_atlases
                .get(&atlas_info.texture_atlas)
                .unwrap()
                .textures[atlas_info.location.glyph_index]
                .as_rect();

            for offset_x in -width..=width {
                // Adjust height to follow a radial pattern.
                let height = ((width_pow2 - offset_x.pow(2)).abs() as f32).sqrt().ceil() as i32;

                for offset_y in -height..=height {
                    if offset_x == 0 && offset_y == 0 {
                        continue;
                    }

                    let offset = Vec2 { x: offset_x as f32, y: offset_y as f32 };

                    let transform = global_transform.affine()
                        * Mat4::from_translation((-0.5 * uinode.size() + offset).extend(0.));

                    let extracted_glyph = ExtractedGlyph {
                        transform: transform * Mat4::from_translation(position.extend(0.)),
                        rect,
                    };

                    if aa_factor != 1.0 && offset_y.abs() == height {
                        aa_glyph_cache.push(extracted_glyph);
                    } else {
                        extracted_uinodes.glyphs.push(extracted_glyph);
                        len += 1;
                    }
                } // y offset
            } // x offset

            if text_layout_info
                .glyphs
                .get(i + 1)
                .is_none_or(|info| info.atlas_info.texture != atlas_info.texture)
            {
                let aa_len = aa_glyph_cache.len();
                for aa_glyph in aa_glyph_cache.drain(..) {
                    extracted_uinodes.glyphs.push(aa_glyph);
                }

                extracted_uinodes.uinodes.push(ExtractedUiNode {
                    render_entity: commands.spawn(TemporaryRenderEntity).id(),
                    stack_index: uinode.stack_index,
                    color,
                    image: atlas_info.texture.id(),
                    clip: clip.map(|clip| clip.clip),
                    extracted_camera_entity,
                    rect,
                    item: ExtractedUiItem::Glyphs { range: start..(start + len) },
                    main_entity: entity.into(),
                });
                start += len;
                len = 0;

                if aa_len > 0 {
                    extracted_uinodes.uinodes.push(ExtractedUiNode {
                        render_entity: commands.spawn(TemporaryRenderEntity).id(),
                        stack_index: uinode.stack_index,
                        color: aa_color,
                        image: atlas_info.texture.id(),
                        clip: clip.map(|clip| clip.clip),
                        extracted_camera_entity,
                        rect,
                        item: ExtractedUiItem::Glyphs { range: start..(start + aa_len) },
                        main_entity: entity.into(),
                    });
                }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_text_shadow_groups(
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    texture_atlases: Extract<Res<Assets<TextureAtlasLayout>>>,
    uinode_query: Extract<
        Query<(
            Entity,
            &ComputedNode,
            &ComputedNodeTarget,
            &GlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            &TextLayoutInfo,
            &TextShadowGroup,
        )>,
    >,
    camera_map: Extract<UiCameraMap>,
)
{
    let mut start = extracted_uinodes.glyphs.len();
    let mut len = 0;

    let mut camera_mapper = camera_map.get_mapper();
    for (entity, uinode, target, global_transform, inherited_visibility, clip, text_layout_info, shadowgroup) in
        &uinode_query
    {
        // Skip if not visible or if size is set to zero (e.g. when a parent is set to `Display::None`)
        if !inherited_visibility.get() || uinode.is_empty() || shadowgroup.len() == 0 {
            continue;
        }

        let Some(extracted_camera_entity) = camera_mapper.map(target) else {
            continue;
        };

        for shadow in shadowgroup.iter() {
            let transform = global_transform.affine()
                * Mat4::from_translation(
                    (-0.5 * uinode.size() + shadow.offset / uinode.inverse_scale_factor()).extend(0.),
                );

            for (i, PositionedGlyph { position, atlas_info, .. }) in text_layout_info.glyphs.iter().enumerate() {
                let rect = texture_atlases
                    .get(&atlas_info.texture_atlas)
                    .unwrap()
                    .textures[atlas_info.location.glyph_index]
                    .as_rect();
                extracted_uinodes.glyphs.push(ExtractedGlyph {
                    transform: transform * Mat4::from_translation(position.extend(0.)),
                    rect,
                });
                len += 1;

                if text_layout_info
                    .glyphs
                    .get(i + 1)
                    .is_none_or(|info| info.atlas_info.texture != atlas_info.texture)
                {
                    extracted_uinodes.uinodes.push(ExtractedUiNode {
                        render_entity: commands.spawn(TemporaryRenderEntity).id(),
                        stack_index: uinode.stack_index,
                        color: shadow.color.into(),
                        image: atlas_info.texture.id(),
                        clip: clip.map(|clip| clip.clip),
                        extracted_camera_entity,
                        rect,
                        item: ExtractedUiItem::Glyphs { range: start..(start + len) },
                        main_entity: entity.into(),
                    });
                    start += len;
                    len = 0;
                }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct UiTextRenderingExtPlugin;

impl Plugin for UiTextRenderingExtPlugin
{
    fn build(&self, app: &mut App)
    {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            ExtractSchedule,
            // Outlines last so they render above shadows.
            (
                extract_text_shadow_groups,
                extract_text_outlines.after(bevy::ui::extract_text_shadows),
            )
                .chain()
                .in_set(RenderUiSystem::ExtractTextShadows),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
