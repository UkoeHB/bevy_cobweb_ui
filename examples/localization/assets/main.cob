#manifest
self as localization

#defs
$title_texture = "images/example_280_220.png"
$text_margin = \ margin:{top:5px bottom: 5px} \

$animation_settings_medium = {duration:0.0265 ease:Linear}
$animation_settings_fast = {duration:0.025 ease:Linear}

#commands
LoadLocalizationManifest{
    default: {
        id: "en-US"
        name: "English"
        manifest: "locales/en-US/main.ftl.ron"
    }
    alts: [
        {
            id: "de-DE"
            name: "German"
            manifest: "locales/de-DE/main.ftl.ron"
        }
        {
            id: "fr-FR"
            name: "French"
            manifest: "locales/fr-FR/main.ftl.ron"
        }
    ]
}
LoadLocalizedFonts[
    {
        family: "Fira Sans"
        attributes: [{weight:Bold}]
        fallbacks: [
            {
                lang: "fr-FR"
                family: "Fira Sans"
                attributes: [{style:Italic}]
            }
            {
                lang: "de-DE"
                family: "Fira Sans"
                attributes: [{style:Italic weight:Bold}]
            }
        ]
    }
]
LoadLocalizedImages[
    {
        image: $title_texture
        fallbacks: [
            {
                lang: "fr-FR"
                image: "images/example_280_220_fr_FR.png"
            }
            {
                lang: "de-DE"
                image: "images/example_280_220_de_DE.png"
            }
        ]
    }
]

#scenes
"root"
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:Center justify_cross:Center}
    BackgroundColor(Hsla{hue:201 saturation:0.4 lightness:0.45 alpha:1})

    "header"
        FlexNode{width:100% min_height:30% justify_main:Center justify_cross:Center}

        "title"
            FlexNode{width:300px}
            LoadedImageNode{image:$title_texture} // Node: ImageMode::Auto auto-sizes the image using its natural aspect ratio.

    "content"
        FlexNode{width:100% flex_direction:Row justify_main:Center justify_cross:Center}

        "selection_section"
            FlexNode{width:50% height:100% flex_direction:Column justify_main:Center justify_cross:Center}

            "selection_box"
                RadioGroup
                FlexNode{
                    padding:{top:8px bottom:8px left:10px right:10px}
                    flex_direction:Column justify_main:SpaceEvenly justify_cross:Center
                }

        "text_section"
            FlexNode{width:50% height:100% left:-5% flex_direction:Column justify_main:Center justify_cross:FlexStart}

            "unlocalized"
                FlexNode{$text_margin}

            "untranslated"
                FlexNode{$text_margin}

            "partially_translated"
                FlexNode{$text_margin}

            "fully_translated"
                FlexNode{$text_margin}

            "font_fallbacks"
                FlexNode{$text_margin}

            "dynamic"
                FlexNode{$text_margin}

            "from_file"
                FlexNode{$text_margin}
                LocalizedText
                TextLine{text:"file-text"}

"selection_button"
    RadioButton // <--- Makes a radio button.
    ControlRoot
    FlexNode{
        min_width:100%
        margin:{top:5px bottom:5px left:5px right:5px}
        flex_direction:Row justify_main:FlexStart justify_cross:Center
    }
    Static<Splat<Border>>{value:2px}
    Animated<Splat<Border>>{state:[Selected] idle:3px enter_idle_with:$animation_settings_fast}
    Static<Splat<Padding>>{value:2px}
    Animated<Splat<Padding>>{state:[Selected] idle:1px enter_idle_with:$animation_settings_fast}
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
            Animated<BackgroundColor>{state:[Selected] idle:#FF000000}

    "text"
        ControlMember
        FlexNode{margin:{top:5px bottom:5px left:10px right:10px}}
        TextLine{size:35}
        TextLineColor(#000000)
