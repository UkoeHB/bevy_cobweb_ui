#scenes
"scene"
    FlexNode{width:100vw height:100vh flex_direction:Row justify_main:SpaceEvenly justify_cross:Center}

    "basic"
        FlexNode{flex_direction:Row justify_main:Center justify_cross:Center}

        "text"
            FlexNode{width:50px margin:{right:15px}}
            TextLine{size:30}

        "slider"
            FlexNode{width:10px height:200px}
            BackgroundColor(#00FF00)
            Slider{axis:Y}

            "handle"
                AbsoluteNode{width:25px height:25px}
                BackgroundColor(#FF5500)
                SliderHandle

    "reverse"
        FlexNode{flex_direction:Row justify_main:Center justify_cross:Center}

        "text"
            FlexNode{width:50px margin:{right:15px}}
            TextLine{size:30}

        "slider"
            FlexNode{width:10px height:200px}
            BackgroundColor(#00FF00)
            Slider{axis:Y direction:Reverse}

            "handle"
                AbsoluteNode{width:25px height:25px}
                BackgroundColor(#FF5500)
                SliderHandle

    "fancy"
        FlexNode{flex_direction:Row justify_main:Center justify_cross:Center}

        "text"
            FlexNode{width:50px margin:{right:15px}}
            TextLine{size:30}

        "slider"
            FlexNode{width:200px height:0px flex_direction:Row justify_main:Center justify_cross:Center}
            Slider{bar_press:Animate{duration:0.25 ease:InOutSine}}

            "start"
                FlexNode{
                    width:20px height:20px
                    left:10px
                    border:{left:2px top:2px bottom:2px}
                }
                BrRadiusTopLeft(11px)
                BrRadiusBottomLeft(11px)
                BackgroundColor(#11BB00)
                BorderColor(#FF5500)

            "bar"
                FlexNode{
                    width:200px height:20px
                    border:{top:2px bottom:2px}
                }
                BackgroundColor(#11BB00)
                BorderColor(#FF5500)

            "end"
                FlexNode{
                    width:20px height:20px
                    left:-10px
                    border:{right:2px top:2px bottom:2px}
                }
                BrRadiusTopRight(11px)
                BrRadiusBottomRight(11px)
                BackgroundColor(#11BB00)
                BorderColor(#FF5500)

            "handle"
                AbsoluteNode{width:0px height:0px justify_main:Center justify_cross:Center}
                SliderHandle

                "handle_view"
                    FlexNode{width:45px height:45px}
                    Splat<Border>(2px)
                    BrRadius(23.5px)
                    BackgroundColor(#550000FF)
                    BorderColor(#FFFFFF)

    "planar"
        FlexNode{flex_direction:Row justify_main:Center justify_cross:Center}

        "text"
            FlexNode{width:150px margin:{right:15px}}
            TextLine{size:30}

        "slider"
            FlexNode{width:200px height:200px}
            BackgroundColor(#00FF00)
            Slider{axis:Planar}

            "handle"
                AbsoluteNode{width:20px height:20px}
                BackgroundColor(#FF5500)
                SliderHandle
