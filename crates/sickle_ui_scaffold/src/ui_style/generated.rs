use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use cob_sickle_macros::StyleCommands;

use crate::*;

/// Derive leaves the original struct, ignore it.
/// (derive macros have a better style overall)
#[derive(StyleCommands)]
enum _StyleAttributes
{
    Display
    {
        display: Display
    },
    PositionType
    {
        position_type: PositionType
    },
    Overflow
    {
        overflow: Overflow
    },
    Left
    {
        left: Val
    },
    Right
    {
        right: Val
    },
    Top
    {
        top: Val
    },
    Bottom
    {
        bottom: Val
    },
    Width
    {
        width: Val
    },
    Height
    {
        height: Val
    },
    MinWidth
    {
        min_width: Val
    },
    MinHeight
    {
        min_height: Val
    },
    MaxWidth
    {
        max_width: Val
    },
    MaxHeight
    {
        max_height: Val
    },
    AspectRatio
    {
        aspect_ratio: Option<f32>
    },
    AlignItems
    {
        align_items: AlignItems
    },
    JustifyItems
    {
        justify_items: JustifyItems
    },
    AlignSelf
    {
        align_self: AlignSelf
    },
    JustifySelf
    {
        justify_self: JustifySelf
    },
    AlignContent
    {
        align_content: AlignContent
    },
    JustifyContent
    {
        justify_content: JustifyContent
    },
    Margin
    {
        margin: UiRect
    },
    Padding
    {
        padding: UiRect
    },
    Border
    {
        border: UiRect
    },
    FlexDirection
    {
        flex_direction: FlexDirection
    },
    FlexWrap
    {
        flex_wrap: FlexWrap
    },
    FlexGrow
    {
        flex_grow: f32
    },
    FlexShrink
    {
        flex_shrink: f32
    },
    FlexBasis
    {
        flex_basis: Val
    },
    RowGap
    {
        row_gap: Val
    },
    ColumnGap
    {
        column_gap: Val
    },
    GridAutoFlow
    {
        grid_auto_flow: GridAutoFlow
    },
    GridTemplateRows
    {
        grid_template_rows: Vec<RepeatedGridTrack>
    },
    GridTemplateColumns
    {
        grid_template_columns: Vec<RepeatedGridTrack>
    },
    GridAutoRows
    {
        grid_auto_rows: Vec<GridTrack>
    },
    GridAutoColumns
    {
        grid_auto_columns: Vec<GridTrack>
    },
    GridRow
    {
        grid_row: GridPlacement
    },
    GridColumn
    {
        grid_column: GridPlacement
    },
    #[target_tupl(BackgroundColor)]
    BackgroundColor
    {
        background_color: Color
    },
    #[target_tupl(BorderColor)]
    BorderColor
    {
        border_color: Color
    },
    #[target_enum]
    FocusPolicy
    {
        focus_policy: FocusPolicy
    },
    #[target_enum]
    Visibility
    {
        visibility: Visibility
    },
    #[skip_enity_command]
    ZIndex
    {
        z_index: ZIndex
    },
    #[skip_ui_style_ext]
    Image
    {
        image: ImageSource
    },
    #[skip_enity_command]
    ImageTint
    {
        image_tint: Color
    },
    #[skip_enity_command]
    ImageFlip
    {
        image_flip: BVec2
    },
    #[skip_enity_command]
    NodeImageMode
    {
        image_scale_mode: Option<NodeImageMode>
    },
    #[static_style_only]
    #[skip_ui_style_ext]
    FluxInteraction
    {
        flux_interaction_enabled: bool
    },
    #[skip_lockable_enum]
    #[skip_ui_style_ext]
    AbsolutePosition
    {
        absolute_position: Vec2
    },
    #[skip_lockable_enum]
    #[skip_enity_command]
    Font
    {
        font: FontSource
    },
    #[skip_lockable_enum]
    #[skip_enity_command]
    FontSize
    {
        font_size: f32
    },
    #[skip_enity_command]
    Scale
    {
        scale: f32
    },
    #[target_enum]
    #[skip_lockable_enum]
    TrackedStyleState
    {
        tracked_style_state: TrackedStyleState
    },
    #[skip_lockable_enum]
    #[skip_enity_command]
    Size
    {
        size: Val
    },
    #[skip_lockable_enum]
    #[target_component(BorderRadius)]
    BorderRadius
    {
        border_radius: BorderRadius
    },
    #[skip_lockable_enum]
    #[target_component(BorderRadius)]
    #[target_component_attr(top_right)]
    BorderTRRadius
    {
        border_tr_radius: Val
    },
    #[skip_lockable_enum]
    #[target_component(BorderRadius)]
    #[target_component_attr(bottom_right)]
    BorderBRRadius
    {
        border_br_radius: Val
    },
    #[skip_lockable_enum]
    #[target_component(BorderRadius)]
    #[target_component_attr(bottom_left)]
    BorderBLRadius
    {
        border_bl_radius: Val
    },
    #[skip_lockable_enum]
    #[target_component(BorderRadius)]
    #[target_component_attr(top_left)]
    BorderTLRadius
    {
        border_tl_radius: Val
    },
    #[skip_lockable_enum]
    #[target_component(Outline)]
    Outline
    {
        outline: Outline
    },
    #[skip_lockable_enum]
    #[target_component(Outline)]
    #[target_component_attr(width)]
    OutlineWidth
    {
        outline_width: Val
    },
    #[skip_lockable_enum]
    #[target_component(Outline)]
    #[target_component_attr(offset)]
    OutlineOffset
    {
        outline_offset: Val
    },
    #[skip_lockable_enum]
    #[target_component(Outline)]
    #[target_component_attr(color)]
    OutlineColor
    {
        outline_color: Color
    },
}
