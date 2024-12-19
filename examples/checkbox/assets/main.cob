#scenes
"scene"
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:SpaceEvenly justify_cross:Center}

    "basic"
        FlexNode{width: 200px flex_direction:Row justify_main:FlexStart justify_cross:Center}

        "checkbox"
            Checkbox // <-- Sets up a checkbox
            ControlRoot
            FlexNode{width:20px height:20px justify_main:Center justify_cross:Center}
            Splat<Border>(2px)
            BackgroundColor(#777777)
            BorderColor(#333333)

            "marker"
                ControlMember
                AbsoluteNode{top:auto left:auto}
                TextLine{text:"x" size:15}
                Multi<Static<DisplayControl>>[{value:Hide} {state:[Checked] value:Show}]

        "text"
            FlexNode{margin:{left:10px}}
            TextLine
