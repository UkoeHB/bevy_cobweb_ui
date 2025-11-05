#import
builtin.colors.tailwind as tw

#defs
$NORMAL_BUTTON = $tw::SLATE_500
$HOVERED_BUTTON = $tw::SLATE_400
$PRESSED_BUTTON = $tw::GREEN_400
$BORDER_BUTTON = $tw::SLATE_400
$BORDER_DISPLAY = $tw::SKY_950
// Defaults to 'button' styling.
+calculator_item = \
    GridNode{ justify_main:Center justify_self_cross:Stretch }
    GridColumn{ span:1 }
    Splat<Border>(1px)
    Splat<Padding>(20px)
    Splat<Margin>(5px)
    BrRadius(5px)
    Splat<BorderColor>($BORDER_BUTTON)
    Responsive<BackgroundColor>{ idle:$NORMAL_BUTTON hover:$HOVERED_BUTTON press:$PRESSED_BUTTON }

    "text"
        FlexNode
        TextLine{ text:"" size:30 }
\

#scenes
"scene"
    GridNode{ grid_template_columns: [(Count(4) auto)] }
    Splat<Margin>(auto)

"button"
    +calculator_item{}

"display"
    +calculator_item{
        GridColumn{ span:3 }
        BrRadius(0px)
        Splat<BorderColor>($BORDER_DISPLAY)
        -Responsive<BackgroundColor>
        BackgroundColor($NORMAL_BUTTON)
    }
