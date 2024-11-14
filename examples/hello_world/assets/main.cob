#manifest
"constants.cob" as constants

#import
constants as const

#defs
$text = "Hello, World!"
$color_anim = \ idle:$const::bgcolor hover:#550055 \

#commands
TestCommand("main")

#scenes
"scene"
    ""
        TextLine{ size: 50.0 text: $text }
        BackgroundColor($const::bgcolor)

    ""
        AbsoluteNode{width:100px height:100px left:500px}
        BackgroundColor($const::bgcolor)

    ""
        AbsoluteNode{width:100px height:100px top:200px}
        Responsive<BackgroundColor>{ $color_anim }
