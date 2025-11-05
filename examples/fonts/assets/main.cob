// Font obtained from https://www.1001freefonts.com/libre-baskerville.font

#commands
RegisterFontFamilies[
    {
        family: "LibreBaskerville"
        fonts: [
            {
                path: "fonts/LibreBaskerville-Regular.ttf"
                width: Normal style: Normal weight: Normal
            }
            {
                path: "fonts/LibreBaskerville-Bold.ttf"
                width: Normal style: Normal weight: Bold
            }
            {
                path: "fonts/LibreBaskerville-Italic.ttf"
                width: Normal style: Italic weight: Normal
            }
        ]
    }
]
LoadFonts["LibreBaskerville"] // <-- required to actually use the font family! Also see LoadLocalizedFonts.

#scenes
"scene"
    FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

    "header"
        FlexNode{border:{bottom:2px} margin:{bottom:8px}}
        Splat<BorderColor>(#FFFFFF)

        // Note: can't have border on a text node
        "text"
            TextLine{text:"Fonts" font:{family:"LibreBaskerville"}}

    "normal"
        TextLine{text:"Normal" font:{family:"LibreBaskerville"}}

    "bold"
        TextLine{text:"Bold" font:{family:"LibreBaskerville" weight:Bold}}

    "italic"
        TextLine{text:"Italic" font:{family:"LibreBaskerville" style:Italic}}
