/*
// Demo

// Basic vertical
- slider bar line (or rectangle)
    - press not animated
    - blocks picking
    - FocusPolicy::Block
    - visible background

    - handle
        - visible background

// Basic vertical reversed
- slider bar line (or rectangle)
    - press not animated
    - blocks picking
    - FocusPolicy::Block
    - visible background

    - handle
        - visible background

// Visible regions detached for alignment
- slider bar line (or rectangle)
    - animate press

    - visible bar centered over slider bar
        - blocks picking
        - FocusPolicy::Block

    - handle dot

        - visible handle centered over dot

// Handle as inset
- slider bar line (or rectangle)
    - animate press
    - blocks picking
    - FocusPolicy::Block
    - visible background

    - handle
        - set width/height
        - visible background

// Discretized values (slider snaps to integer)
- on value change, modify value to equal integer without triggering reactions
    - add custom SliderIntegerValue reactive component that stores the rounded slider value and is used for
    behavior-reactions
- slider bar line (or rectangle)
    - animate press

    - visible bar centered over slider bar
        - blocks picking
        - FocusPolicy::Block

    - handle dot

        - visible handle centered over dot

// Planar
- ...
*/

fn main() {}
