WIP

## Global

Comments, whitespace, and filler characters
- Comments
    - Line comments: `// ... \n`
    - Block comments: `\* ... *\`
- Whitespace: ` `, `\n`
- Filler characters: `,` `;`

Banned characters outside string values and comments
- non-basic ASCII whitespace: `\b`, `\f`, `\r`, `\t`
- any unicode character


## `manifest`

self as {manifest key}
{file} as {manifest key}

### Manifest key

{id}
{id}.{id}.{id}


## `import`

{manifest key} as _
{manifest key} as {alias}

implicit: std as _
    - can override, e.g. 'std as std'

### Alias

{id}


## `using`

{full type path} as {type alias}

### Type alias

{Id}


## `defs`

Definitions
    Value constants
        `${name} = {value}`
        `${name} = \ .. values .. \`
    Value macros
        - 
    Instruction macros
    Scene macros

Invocations
    Value constants
        `${name}`
        `${import::alias::path::to::}{name}`
    Value macros
        - 
    Instruction macros
    Scene macros


## `commands`

Instructions


## `scenes`

Scene layers
- Layer name
    - string value
- Layer stacking
    - add layer stack if encounter layer name >= 2 spaces deeper than current layer
    - if encounter layer name >= 2 spaces shallower than current layer, pop layers until find nearest parent layer at same level
- Layer contents
    - Instructions
    - Instruction macros
    - Scene macros
    - New layers


## Value model

Limitations
- Disallowed struct fields: `auto`, `none`, `nan`, `inf`, `true`, `false`.
- Struct field names must be snake-case
- Instruction names and enum variant names must be camel-case

Instructions
- Identifier
    - Camel-case w/ optional numbers after first letter
    - Optional generics
- Value
    - Unit
    - Struct-like
        - Map container; must start immediately after end of identifier
    - Tuple-like (including newtypes)
        - Tuple container; must start immediately after end of identifier
    - Newtype-of-array
        - Array container; must start immediately after end of identifier

Values
- Containers
- Keywords and special sequences
- Numbers
- Strings
- Bools
    - `true`/`false`

Containers
- Newtype structs and `Option::Some`
    - Implicitly elided
- Maps and structs
    - Delimited by `{ ... }`
    - Map keys can be
        - Struct field names
            - snake-case w/ optional numbers after first letter
        - Values
    - Map values can be
        - Values
- Arrays
    - Delimited by `[ ... ]`
    - Entries can be
        - Values
- Tuples, tuple structs, and unit structs
    - Delimited by `( ... )`
    - Entries can be
        - Values
- Enums
    - Variant identifier
        - Camel-case w/ optional numbers after first letter
    - Value
        - Unit: identifier only (special case: `Option::None` as `none` keyword)
        - Struct-like
            - Map container; must start immediately after end of identifier
        - Tuple-like (including newtype variants of non-array/non-map/non-tuple) (special case: `Option::Some` implicitly elided)
            - Tuple container; must start immediately after end of identifier
        - Newtype-of-array
            - Array container; must start immediately after end of identifier
    - Newtype-variant of tuple/map/array can be flattened (e.g. A({1:1 2:2}) -> A{1:1 2:2})
        - Very handy for newtype-variant of newtype-struct (e.g. Bevy's Color enum can be simplified directly to its variants' inner structs like Srgba{ ... }, where in Rust it looks like Color::Srgba(Srgba{ ... })).

Keywords and special sequences
- `none`
- `true`/`false`
- `nan`/`inf`/`-inf`
- Val variants
    - nums (all floats): `px`, `%`, `vw`, `vh`, `vmin`, `vmax`
        - e.g. `1px` or `5.5%`
    - `auto`
- Hex colors
    - `#` followed by 6 hex digits (upper or lowercase)

Numbers
- Ints deserialize to u128 and i128
- Floats deserialize to f64
    - `nan`/`inf`/`-inf`
    - scientific notation
    - decimals require at least one digit before and after dot

Strings
- Start/end with `"`
- Escape sequences and literals
    - escapes: \b,\f,\n,\r,\t,\",\\,\\u{1 to 6 hex digits}
- Multi-line strings: segment ends in `\` followed by a newline character, next segment begins with first non-space character

Lossy conversions (COB file to rust value back to COB file):
- scientific notation: only floats >= 1e16 or <= 1e-7 will be formatted with scientific notation when serializing to raw COB
- trailing zeroes after decimal in floats: if float can be coerced to int, it will be; otherwise trailing zeroes will be removed other than e.g, `1.0`
- multiline strings: multi-line strings are concatenated
- in-line string formatting (newlines/tabs/etc.) and unicode characters will be replaced with escape sequences
- unicode with leading zeros: leading zeroes removed
- unicode escape sequences will be lower-cased
- hex color sequences will be upper-cased
- manual builtin to auto-builtin
- reflect-defaulted fields: all serializable fields will be serialized
    - workaround: manually filter default values somehow??
- whitespace/comments/filler characters can often, but not always, be recovered using `recover_fill`
