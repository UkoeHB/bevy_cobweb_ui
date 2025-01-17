This module provides extensions to `bevy_ui` that are compatible with COB files and the cobweb animation framework.

These `bevy` types implement [`Instruction`](bevy_cobweb_ui::prelude::Instruction) without any intermediate `bevy_cobweb_ui` types:
- [`BackgroundColor`](bevy::prelude::BackgroundColor): Also implements `StaticAttribute`/`ResponsiveAttribute`/`AnimatedAttribute`.
- [`BorderColor`](bevy::prelude::BorderColor): Also implements `StaticAttribute`/`ResponsiveAttribute`/`AnimatedAttribute`.
- [`ZIndex`](bevy::prelude::ZIndex): Also implements `StaticAttribute`/`ResponsiveAttribute`.
- [`GlobalZIndex`](bevy::prelude::GlobalZIndex): Also implements `StaticAttribute`/`ResponsiveAttribute`.
- [`FocusPolicy`](bevy::ui::FocusPolicy): Also implements `StaticAttribute`/`ResponsiveAttribute`.
- [`Visibility`](bevy::prelude::Visibility): Also implements `StaticAttribute`/`ResponsiveAttribute`.
