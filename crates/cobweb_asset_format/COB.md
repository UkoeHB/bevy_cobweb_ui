Value model comments not fully integrated in the README yet.


Loadables
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
