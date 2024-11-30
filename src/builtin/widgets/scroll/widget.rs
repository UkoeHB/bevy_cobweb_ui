/*

- Scroll

- PreUpdate
    - if content area size changed, need to adjust the slider value so it points at the same location as last tick
        - unless there is an active drag, in which case we just apply the current slider value
    - if content area is within the viewing window minus scroll bars, then disable scroll bars (Display::Hidden + PseudoState::Disabled)
    - if content area changes to be outsize viewing window, re-enable scroll bars
    - if content area changes or viewing window changes, adjust slider bar width to equal Val::Percent(percent in view)

*/
