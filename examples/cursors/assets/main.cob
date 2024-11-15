#commands
PrimaryCursor(Custom{image:"cursor.png" hotspot:(9, 9)})

#scenes
"scene"
    FlexNode{width:100vw height:100vh justify_main:Center justify_cross:Center}

    "box"
        FlexNode{width:200px height:200px justify_main:Center justify_cross:Center}
        BackgroundColor(#00BB00)
        ResponsiveCursor{ hover: System(Move) }

        "inner"
            FlexNode{width:100px height:100px}
            BackgroundColor(#0000FF)
            ResponsiveCursor{ hover: System(Grab), press: System(Grabbing) }
            FocusPolicy::Block // This prevents interactions from reaching the lower node and causing cursor race conditions.
