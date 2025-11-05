use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::sync_world::TemporaryRenderEntity;
use bevy::render::{Extract, RenderApp};
use bevy::text::{PositionedGlyph, TextLayoutInfo};
use bevy::ui_render::{
    stack_z_offsets, ExtractedGlyph, ExtractedUiItem, ExtractedUiNode, ExtractedUiNodes, RenderUiSystems,
    UiCameraMap,
};
use bevy_slow_text_outline::text_outline_rendering::extract_ui_text_outlines;

use super::TextShadowGroup;

//-------------------------------------------------------------------------------------------------------------------

fn extract_text_shadow_groups(
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    texture_atlases: Extract<Res<Assets<TextureAtlasLayout>>>,
    uinode_query: Extract<
        Query<(
            Entity,
            &ComputedNode,
            &ComputedUiTargetCamera,
            &UiGlobalTransform,
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
            let transform = Affine2::from(*global_transform) * Affine2::from_translation(-0.5 * uinode.size());

            for (i, PositionedGlyph { position, atlas_info, .. }) in text_layout_info.glyphs.iter().enumerate() {
                let rect = texture_atlases
                    .get(atlas_info.texture_atlas)
                    .unwrap()
                    .textures[atlas_info.location.glyph_index]
                    .as_rect();
                extracted_uinodes.glyphs.push(ExtractedGlyph {
                    color: shadow.color.into(),
                    translation: position + shadow.offset,
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
                        z_order: uinode.stack_index as f32 + stack_z_offsets::TEXT,
                        image: atlas_info.texture,
                        clip: clip.map(|clip| clip.clip),
                        extracted_camera_entity,
                        transform,
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
            extract_text_shadow_groups
                .before(extract_ui_text_outlines)
                .after(bevy::ui_render::extract_text_shadows)
                .in_set(RenderUiSystems::ExtractTextShadows),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
