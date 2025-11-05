#import
builtin.colors.tailwind as tw

#scenes
"orbiter_widget"
    FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
    FocusPolicy::Block

"field_widget"
    FlexNode{
        margin:{bottom:4px}
        flex_direction:Row justify_main:FlexStart justify_cross:Center
    }

    "name"
        FlexNode{margin:{right:5px}}
        TextLine{size:14}

    "lower_bound"
        FlexNode{margin:{right:6px}}
        TextLine{size:14}

    "value"
        FlexNode{width:65px flex_direction:Row justify_main:Center justify_cross:Center}
        BrRadius(4px)
        Splat<Border>(1px)
        Splat<BorderColor>(#99FFFFFF)
        Responsive<BackgroundColor>{idle:#00000000 hover:#66888888}
        ResponsiveCursor{hover:System(ColResize)}

        "text"
            FlexNode{margin:{top:5px bottom:5px}}
            TextLine{size:14}

    "upper_bound"
        FlexNode{margin:{left:5px}}
        TextLine{size:14}
