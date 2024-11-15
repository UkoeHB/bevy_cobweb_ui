#scenes
"scene"
    FlexNode{
        width:100vw height:100vh
        justify_main:SpaceEvenly justify_cross:Center
    }

    "rectangle"
        FlexNode{
            width:275px height:200px
            justify_main:Center justify_cross:Center
        }
        BackgroundColor(Hsla{ hue:0 saturation:0 lightness:0.4 alpha:1 })
        Animated<PropagateOpacity>{
            idle:0.4
            hover:1
            hover_with:{ duration:0.75 ease:OutExpo delay:0.05 }
            press_with:{ duration:0.75 ease:OutExpo delay:0.01 }
        }

        "inner_rect"
            ControlRoot("Text")
            FlexNode{ min_width:100px min_height:75px }
            BackgroundColor(Hsla{ hue:0 saturation:0 lightness:0.65 alpha:1 })
            Animated<PropagateOpacity>{
                idle:0.4
                hover:1
                hover_with:{ duration:0.75 ease:OutExpo delay:0.05 }
                press_with:{ duration:0.75 ease:OutExpo delay:0.01 }
            }

            "text"
                FlexNode{ margin:{top:10px bottom:10px left:18px right:18px} }

                "inner"
                    TextLine{ size:50 text:"Hover me!" }

                "help_text"
                    // Use a shim to push the help text element into position
                    AbsoluteNode{
                        left:100% bottom:100% right:auto top:auto
                        justify_main:Center justify_cross:Center
                    }

                    "content"
                        ControlLabel("Label")
                        FlexNode{ left:7px bottom:5px right:auto top:auto }
                        BackgroundColor(Hsla{ hue:0 saturation:0 lightness:0 alpha:1 })
                        Animated<PropagateOpacity>{
                            idle:0
                            hover:1
                            hover_with:{ duration:0.05 ease:OutExpo delay:0.5}
                        }

                        "text_elem"
                            FlexNode{ margin:{top:10px bottom:10px left:18px right:18px} }
                            TextLine{ size:20 text:"Nice!" }
