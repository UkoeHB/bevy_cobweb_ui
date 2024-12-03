#defs
$animation_settings_medium = {duration:0.0265, ease:Linear}
$animation_settings_fast = {duration:0.025, ease:Linear},

#scenes
"scene"
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:Center justify_cross:Center}
    BackgroundColor(Hsla{hue:30 saturation:0.09 lightness:0.723 alpha:1})

    "display"
        FlexNode{border:{top:2px bottom:2px left:2px right:2px} justify_main:Center justify_cross:Center}
        BackgroundColor(Hsla{hue:212 saturation:0.33 lightness:0.45 alpha:1})
        BorderColor(Hsla{hue:174 saturation:0.23 lightness:0.18 alpha:1})
        BrRadius(6px)

        "text"
            FlexNode{margin:{top:5px bottom:5px left:9px right:9px}}
            TextLine{size:25}

    "radio_frame"
        RadioButtonGroup // <--- Sets up a radio button group.
        FlexNode{
            margin:{top:20px}
            border:{top:2px bottom:2px left:2px right:2px}
            padding:{top:8px bottom:8px left:10px right:10px}
            flex_direction:Column justify_main:SpaceEvenly justify_cross:Center
        }
        BackgroundColor(Hsla{hue:138 saturation:0.23 lightness:0.57 alpha:1})
        BorderColor(Hsla{hue:174 saturation:0.23 lightness:0.18 alpha:1})
        BrRadius(6px)

// Button widget, implemented from scratch.
"button"
    RadioButton // <--- Makes a radio button.
    ControlRoot
    FlexNode{
        margin:{top:5px bottom:5px left:5px right:5px}
        flex_direction:Row justify_main:FlexStart justify_cross:Center
    }
    Static<Splat<Border>>{value:2px}
    Animated<Splat<Border>>{state:[Selected] enter_ref:2px idle:3px enter_idle_with:$animation_settings_fast}
    Static<Splat<Padding>>{value:2px}
    Animated<Splat<Padding>>{state:[Selected] enter_ref:2px idle:1px enter_idle_with:$animation_settings_fast}
    BrRadius(6px)
    Multi<Animated<BackgroundColor>>[
        {
            idle: Hsla{hue:160 saturation:0.05 lightness:0.94 alpha:1}
            hover: Hsla{hue:192 saturation:0.05 lightness:0.88 alpha:1}
            hover_with: $animation_settings_medium
            press_with: $animation_settings_medium
        }
        {
            state: [Selected]
            enter_ref: Hsla{hue:192 saturation:0.05 lightness:0.88 alpha:1}
            idle: Hsla{hue:197 saturation:0.05 lightness:0.88 alpha:1}
            hover: Hsla{hue:202 saturation:0.05 lightness:0.84 alpha:1}
            enter_idle_with: $animation_settings_medium
            hover_with: $animation_settings_medium
            press_with: $animation_settings_medium
        }
    ]
    BorderColor(Hsla{hue:174 saturation:0.23 lightness:0.18 alpha:1})

    "indicator"
        ControlMember
        FlexNode{
            margin:{top:4px bottom:4px left:9px right:1px}
            justify_main:Center justify_cross:Center
        }
        Splat<Border>(2px)
        BrRadius(8.5px)
        BorderColor(#000000)

        "indicator_dot"
            ControlMember
            FlexNode{width:9px height:9px}
            Splat<Margin>(2px)
            BrRadius(4.5px)
            Static<BackgroundColor>{value:#00000000}
            Animated<BackgroundColor>{
                state: [Selected]
                enter_ref:#00000000
                idle:#FF000000
            }

    "text"
        ControlMember
        FlexNode{
            margin:{top:5px bottom:5px left:10px right:10px}
        }
        TextLine{size:35}
        TextLineColor(#000000)
