mod style_commands;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(
    StyleCommands,
    attributes(
        static_style_only,
        skip_enity_command,
        skip_ui_style_ext,
        skip_lockable_enum,
        animatable,
        target_enum,
        target_tupl,
        target_component,
        target_component_attr,
    )
)]
pub fn style_commands_macro_derive(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    style_commands::derive_style_commands_macro(&ast)
}
