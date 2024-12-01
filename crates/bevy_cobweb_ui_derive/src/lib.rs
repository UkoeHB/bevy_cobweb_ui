mod inner;

use proc_macro::TokenStream;
use syn::DeriveInput;

//-------------------------------------------------------------------------------------------------------------------

#[proc_macro_derive(TypeName)]
pub fn derive_type_name(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_type_name_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable components that can respond to state changes on an entity.
///
/// Implements [`Instruction`] and [`StaticAttribute`] for the type.
///
/// See [`StaticNewtype`] for targeting the inner value of newtype components.
#[proc_macro_derive(StaticComponent)]
pub fn derive_static_component(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_static_component_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable components that can respond to interactions or state changes on the entity.
///
/// Implements [`Instruction`], [`StaticAttribute`], and [`ResponsiveAttribute`].
///
/// See [`ResponsiveNewtype`] for targeting the inner value of newtype components.
#[proc_macro_derive(ResponsiveComponent)]
pub fn derive_responsive_component(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_responsive_component_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable components that can animate in response to interactions or state changes on the entity.
///
/// Implements [`Instruction`], [`StaticAttribute`], [`ResponsiveAttribute`], and [`AnimatedAttribute`].
///
/// See [`AnimatedNewtype`] for targeting the inner value of newtype components.
#[proc_macro_derive(AnimatedComponent)]
pub fn derive_animated_component(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_animated_component_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable reactive components that can respond to state changes on an entity.
///
/// Implements [`Instruction`] and [`StaticAttribute`] for the type.
///
/// See [`StaticReactNewtype`] for targeting the inner value of newtype reactive components.
#[proc_macro_derive(StaticReactComponent)]
pub fn derive_static_react_component(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_static_react_component_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable reactive components that can respond to interactions or state changes on the entity.
///
/// Implements [`Instruction`], [`StaticAttribute`], and [`ResponsiveAttribute`].
///
/// See [`ResponsiveReactNewtype`] for targeting the inner value of newtype reactive components.
#[proc_macro_derive(ResponsiveReactComponent)]
pub fn derive_responsive_react_component(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_responsive_react_component_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable reactive components that can animate in response to interactions or state changes on the
/// entity.
///
/// Implements [`Instruction`], [`StaticAttribute`], [`ResponsiveAttribute`], and [`AnimatedAttribute`].
///
/// See [`AnimatedReactNewtype`] for targeting the inner value of newtype reactive components.
#[proc_macro_derive(AnimatedReactComponent)]
pub fn derive_animated_react_component(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_animated_react_component_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable newtype components whose inner value can respond to state changes on an entity.
///
/// Implements [`Instruction`] and [`StaticAttribute`] for the type.
///
/// See [`StaticComponent`] for targeting the component itself.
#[proc_macro_derive(StaticNewtype)]
pub fn derive_static_newtype(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_static_newtype_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable newtype components whose inner value can respond to interactions or state changes on
/// the entity.
///
/// Implements [`Instruction`], [`StaticAttribute`], and [`ResponsiveAttribute`] for the type.
///
/// See [`ResponsiveComponent`] for targeting the component itself.
#[proc_macro_derive(ResponsiveNewtype)]
pub fn derive_responsive_newtype(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_responsive_newtype_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable newtype components whose inner value can animate in response to interactions or state
/// changes on the entity.
///
/// Implements [`Instruction`], [`StaticAttribute`], [`ResponsiveAttribute`], and [`AnimatedAttribute`] for
/// the type.
///
/// See [`AnimatedComponent`] for targeting the component itself.
#[proc_macro_derive(AnimatedNewtype)]
pub fn derive_animated_newtype(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_animated_newtype_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable newtype reactive components whose inner value can respond to state changes on an entity.
///
/// Implements [`Instruction`] and [`StaticAttribute`] for the type.
///
/// See [`StaticReactComponent`] for targeting the component itself.
#[proc_macro_derive(StaticReactNewtype)]
pub fn derive_static_react_newtype(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_static_react_newtype_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable newtype reactive components whose inner value can respond to interactions or state changes
/// on the entity.
///
/// Implements [`Instruction`], [`StaticAttribute`], and [`ResponsiveAttribute`] for the type.
///
/// See [`ResponsiveReactComponent`] for targeting the component itself.
#[proc_macro_derive(ResponsiveReactNewtype)]
pub fn derive_responsive_react_newtype(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_responsive_react_newtype_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------

/// Derive for loadable newtype reactive components whose inner value can animate in response to interactions or
/// state changes on the entity.
///
/// Implements [`Instruction`], [`StaticAttribute`], [`ResponsiveAttribute`], and [`AnimatedAttribute`] for the
/// type.
///
/// See [`AnimatedReactComponent`] for targeting the component itself.
#[proc_macro_derive(AnimatedReactNewtype)]
pub fn derive_animated_react_newtype(input: TokenStream) -> TokenStream
{
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    inner::derive_animated_react_newtype_impl(ast).into()
}

//-------------------------------------------------------------------------------------------------------------------
