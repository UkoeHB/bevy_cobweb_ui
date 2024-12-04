use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Type};

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_type_name_impl(ast: DeriveInput) -> TokenStream
{
    let struct_name = &ast.ident;
    let type_name = struct_name.to_string();

    // TypeName is not implement for generic types because the full type name is only known when instantiated.
    format!(r#"
        impl TypeName for {} {{
            const NAME: &'static str = "{}";
        }}
    "#, type_name, type_name)
    .parse()
    .unwrap()
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_static_component_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_component_instruction(&ast);
    let static_attr_impl = get_component_static_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_responsive_component_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_component_instruction(&ast);
    let static_attr_impl = get_component_static_attr(&ast);
    let responsive_attr_impl = get_responsive_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
        #responsive_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_animated_component_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_component_instruction(&ast);
    let static_attr_impl = get_component_static_attr(&ast);
    let responsive_attr_impl = get_responsive_attr(&ast);
    let animated_attr_impl = get_component_animated_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
        #responsive_attr_impl
        #animated_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_static_react_component_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_react_component_instruction(&ast);
    let static_attr_impl = get_component_static_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_responsive_react_component_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_react_component_instruction(&ast);
    let static_attr_impl = get_component_static_attr(&ast);
    let responsive_attr_impl = get_responsive_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
        #responsive_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_animated_react_component_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_react_component_instruction(&ast);
    let static_attr_impl = get_component_static_attr(&ast);
    let responsive_attr_impl = get_responsive_attr(&ast);
    let animated_attr_impl = get_react_component_animated_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
        #responsive_attr_impl
        #animated_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_static_newtype_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_component_instruction(&ast);
    let static_attr_impl = get_newtype_static_attr("StaticNewtype", &ast);

    quote! {
        #instruction_impl
        #static_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_responsive_newtype_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_component_instruction(&ast);
    let static_attr_impl = get_newtype_static_attr("ResponsiveNewtype", &ast);
    let responsive_attr_impl = get_responsive_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
        #responsive_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_animated_newtype_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_component_instruction(&ast);
    let static_attr_impl = get_newtype_static_attr("AnimatedNewtype", &ast);
    let responsive_attr_impl = get_responsive_attr(&ast);
    let animated_attr_impl = get_newtype_animated_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
        #responsive_attr_impl
        #animated_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_static_react_newtype_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_react_component_instruction(&ast);
    let static_attr_impl = get_newtype_static_attr("StaticReactNewtype", &ast);

    quote! {
        #instruction_impl
        #static_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_responsive_react_newtype_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_react_component_instruction(&ast);
    let static_attr_impl = get_newtype_static_attr("ResponsiveReactNewtype", &ast);
    let responsive_attr_impl = get_responsive_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
        #responsive_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_animated_react_newtype_impl(mut ast: DeriveInput) -> TokenStream
{
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let instruction_impl = get_react_component_instruction(&ast);
    let static_attr_impl = get_newtype_static_attr("AnimatedReactNewtype", &ast);
    let responsive_attr_impl = get_responsive_attr(&ast);
    let animated_attr_impl = get_react_newtype_animated_attr(&ast);

    quote! {
        #instruction_impl
        #static_attr_impl
        #responsive_attr_impl
        #animated_attr_impl
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_component_instruction(ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    quote!{
        impl #impl_generics Instruction for #struct_name #ty_generics #where_clause
        {
            #[inline(always)]
            fn apply(self, entity: Entity, world: &mut World)
            {
                let Ok(mut emut) = world.get_entity_mut(entity) else { return };
                emut.insert(self);
            }

            #[inline(always)]
            fn revert(entity: Entity, world: &mut World)
            {
                let Ok(mut emut) = world.get_entity_mut(entity) else { return };
                emut.remove::<Self>();
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_react_component_instruction(ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    quote!{
        impl #impl_generics Instruction for #struct_name #ty_generics #where_clause
        {
            #[inline(always)]
            fn apply(self, entity: Entity, world: &mut World)
            {
                let Ok(mut emut) = world.get_entity_mut(entity) else { return };
                match emut.get_mut::<React<Self>>() {
                    Some(mut component) => {
                        *component.get_noreact() = self;
                        React::<Self>::trigger_mutation(entity, world);
                    }
                    None => {
                        world.react(|rc| rc.insert(entity, self));
                    }
                }
            }

            #[inline(always)]
            fn revert(entity: Entity, world: &mut World)
            {
                let Ok(mut emut) = world.get_entity_mut(entity) else { return };
                emut.remove::<React<Self>>();
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_component_static_attr(ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    quote!{
        impl #impl_generics StaticAttribute for #struct_name #ty_generics #where_clause
        {
            type Value = Self;

            #[inline(always)]
            fn construct(value: Self::Value) -> Self
            {
                value
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_newtype_static_attr(name: &'static str, ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;
    let inner_type = match get_newtype_inner_type(name, &ast) {
        Ok(inner_type) => inner_type,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    quote!{
        impl #impl_generics StaticAttribute for #struct_name #ty_generics #where_clause
        {
            type Value = #inner_type;

            #[inline(always)]
            fn construct(value: Self::Value) -> Self
            {
                Self(value)
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_responsive_attr(ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    quote!{
        impl #impl_generics ResponsiveAttribute for #struct_name #ty_generics #where_clause {}
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_component_animated_attr(ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    quote!{
        impl #impl_generics AnimatedAttribute for #struct_name #ty_generics #where_clause
        {
            fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
            {
                let comp = world.get::<Self>(entity)?;
                Some(comp.clone())
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_react_component_animated_attr(ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    quote!{
        impl #impl_generics AnimatedAttribute for #struct_name #ty_generics #where_clause
        {
            fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
            {
                let comp = world.get::<React<Self>>(entity)?;
                Some(comp.get().clone())
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_newtype_animated_attr(ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    quote!{
        impl #impl_generics AnimatedAttribute for #struct_name #ty_generics #where_clause
        {
            fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
            {
                let Self(inner_val) = world.get::<Self>(entity)?;
                Some(inner_val.clone())
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_react_newtype_animated_attr(ast: &DeriveInput) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    quote!{
        impl #impl_generics AnimatedAttribute for #struct_name #ty_generics #where_clause
        {
            fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
            {
                let comp = world.get::<React<Self>>(entity)?;
                let Self(inner_val) = comp.get();
                Some(inner_val.clone())
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_newtype_inner_type<'a>(name: &'static str, ast: &'a DeriveInput) -> syn::Result<&'a Type>
{
    match &ast.data {
        Data::Struct(data_struct) if data_struct.fields.len() == 1 => {
            let field = data_struct.fields.iter().next().unwrap();
            if field.ident.is_some() {
                return Err(syn::Error::new(
                    Span::call_site().into(),
                    format!("{name} can only be derived on newtypes"),
                ));
            }
            Ok(&field.ty)
        }
        _ => Err(syn::Error::new(
            Span::call_site().into(),
            format!("{name} can only be derived on newtypes"),
        )),
    }
}

//-------------------------------------------------------------------------------------------------------------------
