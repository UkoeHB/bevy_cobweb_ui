#scenes
// Custom node structure for the example app.
"root"
    // Root node covers the window.
    FlexNode{ width:100vw height:100vh justify_main:SpaceEvenly justify_cross:Center }

    // Sets up a button with centered content, that animates its background in response to hovers/presses.
    "button"
        ControlRoot("ExampleButton")
        FlexNode{ justify_main:Center justify_cross:Center }
        Animated<BackgroundColor>{ idle:#007000 hover:#006200 press:#005500 }

        // Sets up the button's text as a single line of text with margin to control the edges of the button.
        "text"
            ControlLabel("ExampleButtonText")
            FlexNode{ margin:{top:10px bottom:10px left:18px right:18px} }
            TextLine{ size:50 }
            Animated<TextLineColor>{ idle:#05080F hover:#5080F0 press:#4070E0 }
