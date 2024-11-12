/*
# Manifest

Pattern: ^{{location}}{{any whitespace}}as{{any whitespace}}{{manifest_key}}

Location types:
- self = the current file (for assigning manifest keys to root-level files)
- "path/to/file.cob" = a file in the asset directory

Manifest keys:
- Snake-case only.
- May be separated by '.' at word boundaries.

Behavior:
- Loads the file path (other than `self`) into the app.
- Assigns the manifest key to scenes and importable content from that file.
- Warning emitted if a manifest key is used multiple times in a project.
*/
#manifest
self as experimental
"other/file.cob" as other.file

/*
# Import

Pattern: ^{{location}}{{any whitespace}}as{{any whitespace}}{{local_alias}}

Location types:
- "path/to/file.cob" = a file in the asset directory
- {{manifest_key}} = key for a file loaded by a #manifest somewhere in the dependency graph of this file

Local aliases:
- Snake-case only.
- Use may use '_' to import from the file without assigning an alias.

Behavior:
- Loads file paths into the app if not yet loaded. Does *not* assign a manifest key when doing so.
- Makes all #using from the imported file available to this file.
- Makes all #defs and #specs from the imported file available to this file using the local alias as prefix
(e.g. a constant $colors::tailwind::AMBER_500, or a macro $std::hsla(...), etc.).

Implicit:
- All files implicitly import `std as _` unless you manually include an entry for it, e.g. `std as std`. The
standard import includes various useful macros.
*/
#import
std as std
builtin.widgets.radio_button as radio_button
builtin.colors as colors
"some/file.cob" as _

/*
# Using

Pattern: ^(({{identifier}}::)({{identifier}})*)?{{type_identifier}}\s*[a][s]\s*{{type_identifier}}

Behavior:
- Any reference to the type alias in this file or files that import this file will use the specified type path's
type registration to deserialize values associated with the alias.
*/
#using
my_crate::text::TextLine as TextLine

/*
# Defs

## Data

### Numbers

Pattern: -?(0|[1-9])\d*(?:\.\d+)?(?:[eE][+-]?\d+)
- optional negative
- integer + optional decimal portion (must include a number unlike rust)
- optional scientific notation

Optional unit modifiers: px|%|vw|vh|vmin|vmax
- These convert a number to Val enum variants.

### Color literals

Pattern: (?:#\h{6}(\h{2})?)
- Converts to an Rbga{} color enum variant.

### Language constants

Pattern: true, false, none
- `none` is a distinct constant to avoid interference with None enum variants on user types.

### Strings

Pattern: collect everything between double-parens ".
- Rule: If a line ends in \ then seek forward ignoring whitespace until a character is found.

### Maps

Pattern: collect everything between {}
- A map is a sequence of items.

Map items
- key:value pairs
    - Value options:
        - Data
        - Macro parameter (not catch-all)
- macro 'catch-all' parameter

Map items:
    - 

### Arrays


### Tuples


Implmentation notes:
- Converting to JSON
    - Does deserializing an empty tuple require an empty array or empty string?
    - What about deserializing a unit struct (e.g. if it's somehow a non-defaulted field of a struct)?


## Flatten groups




## Constants

Pattern: ^${{constant_name}}{{any_whitespace}}={{any_whitespace}}{{data}}

Constant names:
- Snake-case only.

Data:
- Allowed:
    (numbers, strings, lang constants, color literals, Val shorthand) = raw data
    CamelCase without <> and with optional {} or () = enum variant
    data macro
    constant
    {} = data map
    [] = data array
    () = tuple
    \\ = flatten inner entries (either map entries or array entries)

Behavior:
- Directly pastes data into another structure.
- Warning emitted if constant overrides another constant or an imported constant.
- Warning emitted if using \\ and the content to paste doesn't match the destination container (map or array).

## Data macros

Similar to a constant, with the addition that parameters can customize part of the data value in the definition.

Pattern: ^${{data_macro_name}}[!][\(]{{params}}[\)]{{any_whitespace}}={{any_whitespace}}{{data}}

Data macro names:
- Snake-case only.

Params:
- normal: uses '{{param_name}}' pattern
    - Optional: add '= {{data}}' to assign a default value.
- catch-all: uses '..{{catch_all_name}}'
    - Name: snake-case
    - Captures all unassigned input items, including anything prefixed with '..', and inserts them as entries
    to a flatten group.
- nested params: uses '{{parent_name}}{..nested params..}' pattern
    - Macro caller uses '{{parent_name}}{..set nested params..}' pattern 

Definition:
- The definition may be a:
    - piece of data
    - data macro call
    - flatten group
- Use '@{{param_name}}' to assign parameter data to a position in the definition.
- Use '?{{param_name}}' to assign optional parameter data to a position in the definition.
    - If the parameter is not set, then if the target is a map value then that map entry will be removed. Otherwise
    the parameter will be ignored.
- Use '..' to assign captured data to wherever a flatten group can be inserted.
    - Array or map (or scene node context for scene macros).

Data:
- Allowed:
    (numbers, strings) = raw data
    CamelCase without <> and with optional {} or () = enum variant
    data macro
    constant
    {} = data map
    [] = data array
    \\ = flatten inner entries (either map entries or array entries)

Behavior:
- Directly pastes data into another structure.
- Warning emitted if constant overrides another constant or an imported constant.
- Warning emitted if using \\ and the content to paste doesn't match the destination container (map or array).

Implementation notes:
- A macro cannot use a 'catch-all' parameter to set parameters of another macro. Instead, it can only assign
parameters (including catch-all parameters) directly to parameters of the other macro. The 'catch-all' is treated
as a distinct value on internal expansion.


## Instruction macros

Similar to data macros, with the addition of:
- The macro definition must be composed of one or more instructions or instruction macros.
- Instruction generics can be set by the macro params.

An instruction is an instruction name plus optional generics plus a data value.


## Scene macros

Similar to instruction macros, with the addition of:
- Definition is implicitly a 'scene node context', which means it can include anything for a normal scene node.
    - Instructions, intruction macros, scene macros, and new scene nodes.
- Camel-case parameters can be used to add to one of the scene node contexts in the definition (i.e. add instructions,
scene macros, scene nodes).
    - Common pattern: parameter like 'InnerNode{..fill}' and then the invoker can set 'InnerNode{..user's instructions..}'
- Unlike data and instruction macros, parameter names must be specified by the user. They are not elided.
- Unlike normal parameters, scene insertion parameters cannot be used to assign data directly. Instead they must have
at least one nesting level, which may contain other scene insertion parameters and one catch-all parameter. Then the user
who makes changes will essentially 'recreate' the node tree at the callsite to access specific nodes.
    - When expanding scene inserters into another scene, allow ..* notation to infer which input parameter/sub-parameter
    to use. Issue a warning if unable to find a parameter sequence in the inputs that matches the parameter sequence
    in the invoked scene.
    - Similarly in the scene definition, infer parameter sequence from node names when ..* notation is used.

Behavior:
- Multiple scene macros can be applied to the same scene node.
    - If two macros insert the same instruction, the *last* insertion will take precedence (i.e. override previous ones).
*/
#defs
/**/$val/**/=/**/220/**/
$boolean = true
$none = none
$aggregate = {x: $val}
// @h means insert 'h' in this position. If 'h' is missing than an error will be emitted.
hsla!(/* in-line comment */ h  s l , a{b{c}} /* over here */) = Hsla<A<B>,@a,?f,C,D bool>{ H!() H()
    hue: @h saturation: @s lightness: ?l alpha: @a.b.c
    AaBb herewego!()
}
ani!(d e) = { duration: @d easing: @e 100.0 $constant #FF000AAA}
// ?t means insert 't' in this position and if 't' is missing then exclude the associated map key.
uirect!(t b l r) = { top: ?t bottom: ?b left: ?l right: ?r }

test!() = $a::b::constant

test!() = a::b::test!()

Test!() = a::b::Test!()

test!(a = none b = 52 c = (true false none auto nan inf -inf)) = MyTuple(@a @b ?c $constant 100.0 (SomeTupleValue 200))

// Only camel-case macros can be used as instructions (aka instruction macro) (only \\ brackets are allowed).
Text!(t s) = TextLine{ text: ?t size: ?s }

animate!(states idle hover press on_over{d e} on_press{d e}) = {
    states: @states
    values: {idle: @idle hover: ?hover press: ?press}
    settings: {pointer_enter: ani!(?on_over.d ?on_over.e) press: ani!(?on_press.d ?on_press.e)}
}
// Macro parameters can have nested parameter lists.
Animate!(type idle{da B df} hover press on_over{d e} on_press{d e}) = Animated<@type>{
    values: {idle: @idle hover: ?hover press: ?press}
    settings: {pointer_enter: ani!(?on_over.d ?on_over.e) press: ani!(?on_press.d ?on_press.e)}
}
// The .. symobol is anonymous and will 'grab all unassigned entries'.
// You can pull the entries into either an array or a map.
MultiAnimate!(type ..) = Multi<Animated<@type>>[..]
// The \ \ brackets mean 'paste all contents in-place' (either map or array entries on the JSON side).
InteractiveRoot!(tag) = \

    ControlRoot(@tag) Interactive

\
F!(w h dir jm jc dm{t b l r = (10)}) = Flex!(
    width:?w height:?h direction:?dir
    justify_main:?jm justify_cross:?jc margin{top:?m.t bottom:?m.b left:?m.l right:?m.r}
)
Nice!() = TheEnd
+basic_spec(
    param1 = 58
    flex{..}
    // Single-quotes means it is inserted to a scene node. These parameters only support .. and nested single-quoted params.
    'inner'{.. 'innermost'{..}}
    // There is no 'catch all' at the base layer because scene macros are inserted in-line to a scene layer.
    // The user can just add stuff directly to the scene layer where the macro is inserted.
) = \
    FlexNode{
        dims: {width: ?param1}
        flex: {..flex}
    }
    BorderColor(#0053AA)
    "inner"
        ..*  // Infer the parameter this refers to from this scene node's name.
        "innermost"
            ..*
\
+spec_override(
    param1
    flex{..}
    'inner'{.. 'innermost'{..}}
) = \
    +basic_spec(
        param1: ?param1
        flex{..flex}
        // We map inputs to this spec into the basic spec's inputs.
        // The parameters these '..*' refer to are inferred from the target parameter names.
        'inner'{..* 'innermost'{..*}}
    )
    +a::b::imported_spec()
    TextLine{ text: "testing multi \
        line text \u{abcd} \n\t\r\\"
    }
\

#commands
LoadImages[
    "a/b.png"
]
AnotherCommand($stuff)
GenericEnum<A B>::Var
A2("aaa")

/*
# Scenes

The scene section must be the *last* section in a cob file.


*/
#scenes
// A spec invocation will bring in parameterized content for the scene.
"scene_from_spec"
    // This one is inserted to the base node
    BackgroundColor($colors::tailwind::AMBER_50)
    +spec_override(
        'inner'{
            // This is inserted to the 'inner' node (named "inner").
            BackgroundColor($colors::tailwind::AMBER_50)
            "add-node"
                ImplicitNode
        }
    )
    A2("aaa")

// Custom node structure for the example app.
// - The node name "root" is optional. An index will be used for its scene path if missing.
"root"
    // Root node covers the window.
    F!(w:Vw($val) h:Vh(100.0) jm:SpaceEvenly jc:Center)

    // Sets up a button with centered content that animates its background in response to hovers/presses.
    "button"
        Flex!(
            justify_main: Center
            justify_cross: Center
        )
        ControlRoot("ExampleButton")
        Animated<BackgroundColor>{
            values: {
                idle: hsla!(274.0 0.25 0.55 0.8)
                hover: hsla!(274.0 0.25 0.55 0.8)
                press: hsla!(274.0 0.25 0.55 0.8)
            }
            settings: {
                pointer_enter: ani!(0.005 OutExpo)
                press: ani!(0.005 InExpo)
            }
        }
        Interactive
        TestMacro!({ x:5 ""} [8 ?param])
        TestEnum::Var(5 10)

        // Sets up the button's text as a single line of text with margin to control the edges of the button.
        "text"
            Flex!(
                // Macro parameters can be marked.
                margin: uirect!(t:10.0% b:10.0% l:18.0px r:18.0px)
            )
            ControlLabel("ExampleButtonText")
            // Using marked parameters lets you skip other parameters. This text has no text because it is dynamically
            // updated in-code.
            Text!(s: 50.0)
            Animated<TextLineColor>{
                values: {
                    idle: hsla!(0.0 0.15 1.0 1.0)
                    hover: hsla!(0.0 0.23 0.9 1.0)
                    press: hsla!(0.0 0.31 0.8 1.0)
                }
                settings: {
                    pointer_enter: ani!(0.15 OutExpo)
                    press: ani!(0.2 OutExp)
                }
            }
            Multi<Animated<DimsTop>>[
                { values: {idle: Px(20.0)}, settings: {pointer_enter: ani!(0.3 InExpo)} }
                { states: [Selected], values: {idle: Px(10.0)}, settings: {pointer_enter: ani!(0.3 InExpo)} }
            ]

"concise"
    F!(100vw 100vh Row SpaceEvenly Center)

    "button"
        F!(jm:Center jc:Center)
        InteractiveRoot!("ExampleButton")
        Animate!(BackgroundColor
            hsla!(274 0.25 0.55 0.8) h:hsla!(274 0.25 0.55 0.8) p:hsla!(274 0.25 0.55 0.8)
            on_over{$some_dur OutExp} on_press{0.005 InExpo})

        "txt"
            F!(m{10% 10% 18px 18px})
            ControlLabel("ExampleButtonText")
            Text!(s: 50.0)
            Animate!(TextLineColor #000000 h:#112345 p:#F01111AA on_over{0.15 OutExpo} on_press{0.2 OutExpo})
            MultiAnimate!(DimsTop
                animate!([] 20px h:30px on_over{0.3 InExpo})
                animate!([Selected] 20px h:30px on_over{0.3 InExpo}))

            "inserted"
                +spec_in_tree(num: 42)

            "another"
                F!(10px 11px)
                Text!("Hello! \"{user}\"")
