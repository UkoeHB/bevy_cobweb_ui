use bevy::prelude::*;
use bevy::render::sync_world::TemporaryRenderEntity;
use bevy::render::{Extract, RenderApp};
use bevy::text::{PositionedGlyph, TextLayoutInfo};
use bevy::ui::{ExtractedGlyph, ExtractedUiItem, ExtractedUiNode, ExtractedUiNodes, RenderUiSystem, UiCameraMap};

use super::{TextOutline, TextShadowGroup};

//-------------------------------------------------------------------------------------------------------------------

fn extract_text_outlines(
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
    let mut start = extracted_uinodes.glyphs.len();
    let mut end = start + 1;

    let mut camera_mapper = camera_map.get_mapper();
    for (entity, uinode, target, global_transform, inherited_visibility, clip, text_layout_info, outline) in
        &uinode_query
    {
        // Skip if not visible or if size is set to zero (e.g. when a parent is set to `Display::None`)
        if !inherited_visibility.get() || uinode.is_empty() {
            continue;
        }

        let Some(extracted_camera_entity) = camera_mapper.map(target) else {
            continue;
        };

        for offset_x in (-outline.width as i32)..=(outline.width as i32) {
            for offset_y in (-outline.width as i32)..=(outline.width as i32) {
                // Don't apply extra corner glyphs if using soft corners.
                if outline.soft_corners
                    && (offset_x.abs() == offset_y.abs())
                    && (offset_x.abs() == outline.width as i32)
                {
                    continue;
                }

                if offset_x == 0 && offset_y == 0 {
                    continue;
                }

                let offset = Vec2 { x: offset_x as f32, y: offset_y as f32 };

                let transform = global_transform.affine()
                    * Mat4::from_translation(
                        (-0.5 * uinode.size() + offset / uinode.inverse_scale_factor()).extend(0.),
                    );

                for (i, PositionedGlyph { position, atlas_info, span_index, .. }) in
                    text_layout_info.glyphs.iter().enumerate()
                {
                    let rect = texture_atlases
                        .get(&atlas_info.texture_atlas)
                        .unwrap()
                        .textures[atlas_info.location.glyph_index]
                        .as_rect();
                    extracted_uinodes.glyphs.push(ExtractedGlyph {
                        transform: transform * Mat4::from_translation(position.extend(0.)),
                        rect,
                    });

                    if text_layout_info.glyphs.get(i + 1).is_none_or(|info| {
                        info.span_index != *span_index || info.atlas_info.texture != atlas_info.texture
                    }) {
                        extracted_uinodes.uinodes.push(ExtractedUiNode {
                            render_entity: commands.spawn(TemporaryRenderEntity).id(),
                            stack_index: uinode.stack_index,
                            color: outline.color.into(),
                            image: atlas_info.texture.id(),
                            clip: clip.map(|clip| clip.clip),
                            extracted_camera_entity,
                            rect,
                            item: ExtractedUiItem::Glyphs { range: start..end },
                            main_entity: entity.into(),
                        });
                        start = end;
                    }

                    end += 1;
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
    let mut end = start + 1;

    let mut camera_mapper = camera_map.get_mapper();
    for (entity, uinode, target, global_transform, inherited_visibility, clip, text_layout_info, shadowgroup) in
        &uinode_query
    {
        // Skip if not visible or if size is set to zero (e.g. when a parent is set to `Display::None`)
        if !inherited_visibility.get() || uinode.is_empty() {
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

            for (i, PositionedGlyph { position, atlas_info, span_index, .. }) in
                text_layout_info.glyphs.iter().enumerate()
            {
                let rect = texture_atlases
                    .get(&atlas_info.texture_atlas)
                    .unwrap()
                    .textures[atlas_info.location.glyph_index]
                    .as_rect();
                extracted_uinodes.glyphs.push(ExtractedGlyph {
                    transform: transform * Mat4::from_translation(position.extend(0.)),
                    rect,
                });

                if text_layout_info.glyphs.get(i + 1).is_none_or(|info| {
                    info.span_index != *span_index || info.atlas_info.texture != atlas_info.texture
                }) {
                    extracted_uinodes.uinodes.push(ExtractedUiNode {
                        render_entity: commands.spawn(TemporaryRenderEntity).id(),
                        stack_index: uinode.stack_index,
                        color: shadow.color.into(),
                        image: atlas_info.texture.id(),
                        clip: clip.map(|clip| clip.clip),
                        extracted_camera_entity,
                        rect,
                        item: ExtractedUiItem::Glyphs { range: start..end },
                        main_entity: entity.into(),
                    });
                    start = end;
                }

                end += 1;
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
                bevy::ui::extract_text_shadows,
                extract_text_shadow_groups,
                extract_text_outlines,
            )
                .chain()
                .in_set(RenderUiSystem::ExtractTextShadows),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
