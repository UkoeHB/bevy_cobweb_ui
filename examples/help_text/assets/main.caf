#scenes
"scene"
    FlexStyle{
        dims:{width:100vw , height:100vh},
        content:{justify_main: SpaceEvenly, justify_cross: Center},
    }
    "rectangle"
        FlexStyle{
            dims: {width: 275px, height: 200px},
            content: {justify_main: Center, justify_cross: Center}
        }
        BackgroundColor(Hsla{hue: 0, saturation: 0, lightness: 0.4, alpha: 1})
        Animated<PropagateOpacity>{
            values: {
                idle: 0.4
                hover: 1
            }
            settings: {
                pointer_enter: { duration: 0.75 easing: OutExpo delay: 0.05}
                press: { duration: 0.75 easing: OutExpo delay: 0.01}
            }
        }
        "inner_rect"
            ControlRoot("Text")
            FlexStyle{
                dims: {min_width: 100px, min_height: 75px},
            }
            BackgroundColor(Hsla{hue: 0, saturation: 0, lightness: 0.65, alpha: 1})
            Animated<PropagateOpacity>{
                values: {
                    idle: 0.4
                    hover: 1
                }
                settings: {
                    pointer_enter: { duration: 0.75 easing: OutExpo delay: 0.05}
                    press: { duration: 0.75 easing: OutExpo delay: 0.01}
                }
            }
            "text"
                FlexStyle{
                    flex: {margin: {top: 10px, bottom: 10px, left: 18px, right: 18px}},
                },

                "inner"
                    TextLine{ size: 50 text: "Hover me!" },

                "help_text"
                    // Use a shim to push the help text element into position
                    AbsoluteStyle{
                        dims:{
                            left: 100%, bottom: 100%, right: auto, top: auto
                        },
                        content: {justify_main: Center, justify_cross: Center}
                    }
                    "content"
                        ControlLabel("Label")
                        FlexStyle{
                            dims: {left:7px, bottom:5px, right: auto, top: auto},
                        }
                        BackgroundColor(Hsla{hue: 0, saturation: 0, lightness: 0, alpha: 1}),
                        Animated<PropagateOpacity>{
                            values:{
                                idle: 0,
                                hover: 1
                            }
                            settings: {
                                pointer_enter: { duration: 0.05 easing: OutExpo delay: 0.5}
                            }
                        }
                        "text_elem"
                            FlexStyle{
                                flex: {margin: {top: 10px, bottom: 10px, left: 18px, right: 18px}},
                            },
                            TextLine{ size: 20 text: "Nice!" },
