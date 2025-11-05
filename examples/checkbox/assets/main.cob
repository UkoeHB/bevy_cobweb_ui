#defs
$toggle_animation = {duration:0.2 ease:InOutSine}

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
            Splat<BorderColor>(#333333)

            "marker"
                ControlMember
                AbsoluteNode{top:auto left:auto}
                TextLine{text:"x" size:15}
                Multi<Static<DisplayControl>>[{value:Hide} {state:[Checked] value:Show}]

        "text"
            FlexNode{margin:{left:10px}}
            TextLine

    "toggle"
        FlexNode{width: 200px flex_direction:Row justify_main:FlexStart justify_cross:Center}

        "checkbox"
            Checkbox // <-- Sets up a checkbox
            ControlRoot
            FlexNode{width:70px height:35px justify_main:Center justify_cross:Center}
            BrRadius(35px)
            Splat<Border>(2px)
            Multi<Animated<BackgroundColor>>[
                {idle:#777777 enter_idle_with:$toggle_animation delete_on_entered:true}
                {state:[Checked] idle:#3333FF enter_idle_with:$toggle_animation delete_on_entered:true}
            ]
            Splat<BorderColor>(#333333)

            // Channel for the marker to move in.
            "channel"
                FlexNode{width:35px flex_direction:Row justify_main:Center justify_cross:Center}

                // Anchor for the marker. The anchor slides in the channel.
                "anchor"
                    ControlMember
                    AbsoluteNode{top:auto justify_main:Center justify_cross:Center}
                    Multi<Animated<DimsLeft>>[
                        {idle:0% enter_idle_with:$toggle_animation delete_on_entered:true}
                        {state:[Checked] idle:100% enter_idle_with:$toggle_animation delete_on_entered:true}
                    ]

                    // Marker centered on top of the anchor.
                    "marker"
                        AbsoluteNode{top:auto left:auto width:25px height:25px}
                        BrRadius(25px)
                        BackgroundColor(#33FF33)
