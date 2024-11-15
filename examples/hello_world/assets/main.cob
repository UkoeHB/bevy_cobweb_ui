#scenes
"scene"
    FlexNode{
        padding:{top:2px bottom:2px left:2px right:2px}
        border:{top:8px left:8px right:8px bottom:8px}
    }
    BorderColor(#C60000)
    Animated<BackgroundColor>{idle:#009900 hover:#00AA00}

    "text"
        TextLine{ text: "Hello, World!" size: 50 }
        Animated<TextLineColor>{ idle:#00D070 hover:#00F070 }
