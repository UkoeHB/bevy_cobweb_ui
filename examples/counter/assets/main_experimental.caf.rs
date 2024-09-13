/*
# Syntax notes

## Scene node prefix expansion

Expand into builtin and known macros.

// Shorthand
#scene FLEX "root" {
    width: Vw(100.0)
}

// Expanded
#scene "root" {
    flex!(width:Vw(100.0))
}
*/

// Pre-registered macros.
#macros {
    // @h means insert 'h' in this position. If 'h' is missing than an error will be emitted.
    hsla!(h, s, l, a) => Hsla{ hue: @h, saturation: @s, lightness: @l, alpha: @a },
    easing!(d, e) => { duration: @d, easing: @e },
    // ?t means insert 't' in this position and if 't' is missing then exclude the associated map key.
    uirect!(t, b, l, r) => { top: ?t, bottom: ?b, left: ?l, right: ?r },
    // Macros can be invoked as loadables or from inside loadable definitions
    text!(t, s) => TextLine{ text: ?t, size: ?s },
}

// Custom node structure for the example app.
// - The node name "root" is optional. An index will be used for its scene path if missing.
#scene FLEX "root" {
    // Root node covers the window.
    width: Vw(100.0),
    height: Vh(100.0),
    justify_main: SpaceEvenly,
    justify_cross: Center,

    // Sets up a button with centered content, that animates its background in response to hovers/presses.
    FLEX "button" {
        justify_main: Center,
        justify_cross: Center,
        ControlRoot("ExampleButton"),
        Animated<BgColor> {
            values: {
                idle: hsla!(274.0, 0.25, 0.55, 0.8),
                hover: hsla!(274.0, 0.25, 0.55, 0.8),
                press: hsla!(274.0, 0.25, 0.55, 0.8)
            },
            settings: {
                pointer_enter: easing!(0.005, OutExpo),
                press: easing!(0.005, InExpo),
            }
        },
        Interactive,

        // Sets up the button's text as a single line of text with margin to control the edges of the button.
        FLEX "text" {
            // Macro parameters can be marked.
            margin: uirect!(t:Px(10.0), b:Px(10.0), l:Px(18.0), r:Px(18.0)),
            ControlLabel("ExampleButtonText"),
            // Using marked parameters lets you skip other parameters. This text has no text because it is dynamically
            // updated in-code.
            text!(s: 50.0),
            Animated<TextLineColor> {
                values: {
                    idle: hsla!(0.0, 0.15, 1.0, 1.0),
                    hover: hsla!(0.0, 0.23, 0.9, 1.0),
                    press: hsla!(0.0, 0.31, 0.8, 1.0)
                },
                settings: {
                    pointer_enter: easing!(0.15, OutExpo),
                    press: easing!(0.2, OutExp)}
                }
            },
        }
    }
}
