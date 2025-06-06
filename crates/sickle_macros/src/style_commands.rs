use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{AttrStyle, Attribute, Data, DataEnum, Fields, Meta, Type, TypePath, Variant};

#[derive(Clone, Copy, Debug)]
enum ParseError
{
    InvalidVariant,
    NoFields,
    TooManyFields,
    InvalidType,
    InvalidTargetTuplType,
    InvalidTargetComponentType,
    InvalidTargetComponentAttrType,
}

#[derive(Clone, Debug)]
struct StyleAttribute
{
    ident: Ident,
    command: Ident,
    type_path: TypePath,
    target_tupl: Option<proc_macro2::TokenStream>,
    target_component: Option<proc_macro2::TokenStream>,
    target_component_attr: Option<Ident>,
    target_enum: bool,
    static_style_only: bool,
    skip_enity_command: bool,
    skip_ui_style_ext: bool,
    skip_lockable_enum: bool,
    cmd_struct_name: String,
    cmd_struct_ident: Ident,
    target_attr_name: String,
}

impl StyleAttribute
{
    fn new(ident: Ident, command: Ident, type_path: TypePath) -> Self
    {
        let cmd_struct_name = format!("Set{}", ident);
        let cmd_struct_ident = Ident::new(cmd_struct_name.as_str(), ident.span().clone());
        let target_attr_name = command.to_string();

        Self {
            ident,
            command,
            type_path,
            target_tupl: None,
            target_component: None,
            target_component_attr: None,
            target_enum: false,
            static_style_only: false,
            skip_enity_command: false,
            skip_ui_style_ext: false,
            skip_lockable_enum: false,
            cmd_struct_name,
            cmd_struct_ident,
            target_attr_name,
        }
    }
}

pub(crate) fn derive_style_commands_macro(ast: &syn::DeriveInput) -> TokenStream
{
    let name_ident = &ast.ident;
    let Data::Enum(enum_data) = &ast.data else {
        return quote_spanned! {
            name_ident.span() => compile_error!("Invalid template type: Must be an enum!");
        }
        .into();
    };

    let attributes = match parse_variants(enum_data) {
        Ok(attributes) => attributes,
        Err((span, error)) => return match_error(span, error).into(),
    };

    let stylable_attribute = prepare_stylable_attribute(&attributes);
    let lockable_attribute = prepare_lockable_attribute(&attributes);
    let static_style_attribute = prepare_static_style_attribute(&attributes);
    let style_commands = prepare_style_commands(&attributes);

    quote! {
        #static_style_attribute
        #lockable_attribute
        #stylable_attribute
        #style_commands
    }
    .into()
}

fn match_error(span: proc_macro2::Span, error: ParseError) -> proc_macro2::TokenStream
{
    match error {
        ParseError::InvalidVariant => {
            return quote_spanned! {
                span => compile_error!("Invlaid variant: Must be a struct with named fields");
            }
        }
        ParseError::NoFields => {
            return quote_spanned! {
                span => compile_error!("No fields defined");
            }
        }
        ParseError::TooManyFields => {
            return quote_spanned! {
                span => compile_error!("Too many fields");
            }
        }
        ParseError::InvalidType => {
            return quote_spanned! {
                span => compile_error!("Invalid Type: Must be a TypePath");
            }
        }
        ParseError::InvalidTargetTuplType => {
            return quote_spanned! {
                span => compile_error!("Unsupported target_tupl value. Must be defined as #[target_tupl(Component)]");
            }
        }
        ParseError::InvalidTargetComponentType => {
            return quote_spanned! {
                span => compile_error!("Unsupported target_component value. Must be defined as #[target_component(Component)]");
            }
        }
        ParseError::InvalidTargetComponentAttrType => {
            return quote_spanned! {
                span => compile_error!("Unsupported target_component_attr value. Must be defined as #[target_component_attr(attr)]. Must be used along with target_component.");
            }
        }
    }
}

fn parse_variants(data: &DataEnum) -> Result<Vec<StyleAttribute>, (proc_macro2::Span, ParseError)>
{
    let attributes: Result<Vec<_>, _> = data.variants.iter().map(parse_variant).collect();
    attributes
}

fn parse_variant(variant: &Variant) -> Result<StyleAttribute, (proc_macro2::Span, ParseError)>
{
    let variant_ident = variant.ident.clone();

    let Fields::Named(fields) = variant.fields.clone() else {
        return Err((variant.span(), ParseError::InvalidVariant));
    };
    if fields.named.len() == 0 {
        return Err((variant.span(), ParseError::NoFields));
    }
    if fields.named.len() > 1 {
        return Err((variant.span(), ParseError::TooManyFields));
    }

    // Safe unwrap, we checked above that it extists
    let field = fields.named.first().unwrap();
    let Some(command) = field.ident.clone() else {
        return Err((field.ty.span(), ParseError::InvalidVariant));
    };

    let Type::Path(attr_type) = field.ty.clone() else {
        return Err((field.ty.span(), ParseError::InvalidType));
    };

    let mut attribute = StyleAttribute::new(variant_ident, command, attr_type);

    for attr in &variant.attrs {
        if attr.style == AttrStyle::Outer {
            if attr.path().is_ident("target_enum") {
                attribute.target_enum = true;
            } else if attr.path().is_ident("static_style_only") {
                attribute.static_style_only = true;
            } else if attr.path().is_ident("skip_enity_command") {
                attribute.skip_enity_command = true;
            } else if attr.path().is_ident("skip_ui_style_ext") {
                attribute.skip_ui_style_ext = true;
            } else if attr.path().is_ident("skip_lockable_enum") {
                attribute.skip_lockable_enum = true;
            } else if attr.path().is_ident("target_tupl") {
                let token_stream = target_component(attr, ParseError::InvalidTargetTuplType)?;
                attribute.target_tupl = Some(token_stream);
            } else if attr.path().is_ident("target_component") {
                let token_stream = target_component(attr, ParseError::InvalidTargetComponentType)?;
                attribute.target_component = Some(token_stream);
            } else if attr.path().is_ident("target_component_attr") {
                let component_attr_ident = target_component_attr(attr)?;
                attribute.target_component_attr = Some(component_attr_ident);
            }
        }
    }

    Ok(attribute)
}

fn target_component(
    attr: &Attribute,
    error: ParseError,
) -> Result<proc_macro2::TokenStream, (proc_macro2::Span, ParseError)>
{
    let attr_span = attr.path().get_ident().unwrap().span();
    let Meta::List(list) = &attr.meta else {
        return Err((attr_span, error));
    };

    if list.tokens.is_empty() {
        return Err((attr_span, error));
    }

    let tokens = list.tokens.clone().into_iter();
    let has_invalid_parts = tokens.clone().any(|e| match e {
        proc_macro2::TokenTree::Group(_) => true,
        proc_macro2::TokenTree::Ident(_) => false,
        proc_macro2::TokenTree::Punct(_) => false,
        proc_macro2::TokenTree::Literal(_) => true,
    });

    if tokens.clone().count() == 0 || has_invalid_parts {
        return Err((attr_span, error));
    }

    Ok(list.tokens.clone())
}

fn target_component_attr(attr: &Attribute) -> Result<Ident, (proc_macro2::Span, ParseError)>
{
    let attr_span = attr.path().get_ident().unwrap().span();
    let Meta::List(list) = &attr.meta else {
        return Err((attr_span, ParseError::InvalidTargetComponentAttrType));
    };

    if list.tokens.is_empty() {
        return Err((attr_span, ParseError::InvalidTargetComponentAttrType));
    }

    // MetaList {
    //     path: Path {
    //         leading_colon: None,
    //         segments: [PathSegment {
    //             ident: Ident {
    //                 ident: "target_component_attr",
    //                 span: SpanData {
    //                     range: 4549..4570,
    //                     anchor: SpanAnchor(FileId(13048), 5),
    //                     ctx: SyntaxContextId(0),
    //                 },
    //             },
    //             arguments: PathArguments::None,
    //         }],
    //     },
    //     delimiter: MacroDelimiter::Paren(Paren),
    //     tokens: TokenStream[Ident {
    //         ident: "top_left",
    //         span: SpanData {
    //             range: 4571..4579,
    //             anchor: SpanAnchor(FileId(13048), 5),
    //             ctx: SyntaxContextId(0),
    //         },
    //     }],
    // };

    if let Some(attr_ident) = list.tokens.clone().into_iter().find(|e| match e {
        proc_macro2::TokenTree::Ident(_) => true,
        _ => false,
    }) {
        match attr_ident {
            proc_macro2::TokenTree::Ident(attr_ident) => Ok(attr_ident),
            _ => unreachable!(),
        }
    } else {
        return Err((attr_span, ParseError::InvalidTargetComponentAttrType));
    }
}

fn prepare_stylable_attribute(style_attributes: &Vec<StyleAttribute>) -> proc_macro2::TokenStream
{
    let base_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .map(to_base_attribute_variant)
        .collect();

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
        pub enum StylableAttribute {
            #(#base_variants)*
        }
    }
}

fn prepare_lockable_attribute(style_attributes: &Vec<StyleAttribute>) -> proc_macro2::TokenStream
{
    let base_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .filter(|v| !v.skip_lockable_enum)
        .map(to_base_attribute_variant)
        .collect();

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
        pub enum LockableStyleAttribute {
            #(#base_variants)*
        }
    }
}

fn prepare_static_style_attribute(style_attributes: &Vec<StyleAttribute>) -> proc_macro2::TokenStream
{
    let variants = style_attributes.iter();
    let base_variants: Vec<proc_macro2::TokenStream> = variants.clone().map(to_static_style_variant).collect();
    let eq_variants: Vec<proc_macro2::TokenStream> = variants.clone().map(to_eq_style_variant).collect();
    let apply_variants: Vec<proc_macro2::TokenStream> = variants
        .clone()
        .map(to_static_style_apply_variant)
        .collect();
    let builder_fns: Vec<proc_macro2::TokenStream> = variants.clone().map(to_static_style_builder_fn).collect();

    quote! {
        #[derive(Clone, Debug)]
        pub enum StaticStyleAttribute {
            #(#base_variants)*
            Custom(CustomStaticStyleAttribute),
        }

        impl LogicalEq for StaticStyleAttribute {
            fn logical_eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #(#eq_variants)*
                    (Self::Custom(l0), Self::Custom(r0)) => l0.logical_eq(r0),
                    _ => false,
                }
            }
        }

        impl StaticStyleAttribute {
            pub fn apply(&self, ui_style: &mut UiStyle) {
                match self {
                    #(#apply_variants)*
                    Self::Custom(attr) => {
                        attr.apply(ui_style);
                    }
                }
            }
        }

        impl StyleBuilder {
            #(#builder_fns)*
        }
    }
}

fn prepare_style_commands(style_attributes: &Vec<StyleAttribute>) -> proc_macro2::TokenStream
{
    let extensions: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .filter(|v| !v.skip_ui_style_ext)
        .map(to_ui_style_extensions)
        .collect();

    let implementations: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .filter(|v| !(v.skip_ui_style_ext || v.skip_enity_command))
        .map(to_ui_style_command_impl)
        .collect();

    quote! {
        #(#extensions)*
        #(#implementations)*
    }
}

fn to_eq_style_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream
{
    let ident = &style_attribute.ident;
    quote! {
        (Self::#ident(_), Self::#ident(_)) => true,
    }
}

fn to_base_attribute_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream
{
    let ident = &style_attribute.ident;
    quote! {
        #ident,
    }
}

fn to_static_style_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream
{
    let ident = &style_attribute.ident;
    let type_path = &style_attribute.type_path;
    quote! {
        #ident(#type_path),
    }
}

fn to_static_style_apply_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream
{
    let ident = &style_attribute.ident;
    let command = &style_attribute.command;
    quote! {
        Self::#ident(value) => {
            ui_style.#command(value.clone());
        }
    }
}

fn to_static_style_builder_fn(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream
{
    let ident = &style_attribute.ident;
    let type_path = &style_attribute.type_path;
    let command = &style_attribute.command;
    quote! {
        pub fn #command(&mut self, #command: impl Into<#type_path>) -> &mut Self {
            self.add(DynamicStyleAttribute::Static(
                StaticStyleAttribute::#ident(#command.into()),
            ));

            self
        }
    }
}

fn to_ui_style_extensions(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream
{
    let cmd_struct_name = &style_attribute.cmd_struct_name.clone();
    let cmd_struct_ident = &style_attribute.cmd_struct_ident.clone();
    let target_attr = &style_attribute.command;
    let target_type = &style_attribute.type_path;

    let extension_name = String::from(cmd_struct_name.clone()) + "Ext";
    let extension_ident = Ident::new(extension_name.as_str(), cmd_struct_ident.span().clone());
    let extension_unchecked_name = String::from(cmd_struct_name.as_str()) + "UncheckedExt";
    let extension_unchecked_ident = Ident::new(extension_unchecked_name.as_str(), cmd_struct_ident.span().clone());

    quote! {
        pub struct #cmd_struct_ident {
            pub #target_attr: #target_type,
            pub check_lock: bool,
        }

        pub trait #extension_ident {
            fn #target_attr(&mut self, #target_attr: #target_type) -> &mut Self;
        }

        impl #extension_ident for UiStyle<'_> {
            fn #target_attr(&mut self, #target_attr: #target_type) -> &mut Self {
                self.entity_commands().queue(#cmd_struct_ident {
                    #target_attr,
                    check_lock: true
                });
                self
            }
        }

        pub trait #extension_unchecked_ident {
            fn #target_attr(&mut self, #target_attr: #target_type) -> &mut Self;
        }

        impl #extension_unchecked_ident for UiStyleUnchecked<'_> {
            fn #target_attr(&mut self, #target_attr: #target_type) -> &mut Self {
                self.entity_commands().queue(#cmd_struct_ident {
                    #target_attr,
                    check_lock: false
                });
                self
            }
        }
    }
}

fn to_ui_style_command_impl(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream
{
    let ident = &style_attribute.ident;
    let cmd_struct_ident = &style_attribute.cmd_struct_ident.clone();
    let target_attr_name = &style_attribute.target_attr_name;
    let setter = to_setter_entity_command_frag(style_attribute);

    let check_lock = match style_attribute.skip_lockable_enum {
        true => proc_macro2::TokenStream::new(),
        false => quote! {
            if self.check_lock {
                if let Some(locked_attrs) = world.get::<LockedStyleAttributes>(entity) {
                    if locked_attrs.contains(LockableStyleAttribute::#ident) {
                        warn!(
                            "Failed to style {} property on entity {}: Attribute locked!",
                            #target_attr_name,
                            entity
                        );
                        return;
                    }
                }
            }
        },
    };

    quote! {
        impl EntityCommand for #cmd_struct_ident {
            fn apply(self, mut e: EntityWorldMut)
            {
                let entity = e.id();
                let world = e.into_world_mut();
                #check_lock
                #setter
            }
        }
    }
}

fn to_setter_entity_command_frag(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream
{
    let target_attr = &style_attribute.command;
    let target_type = &style_attribute.type_path;
    let target_attr_name = &style_attribute.target_attr_name;

    if style_attribute.target_enum {
        let target_type_name = target_type.path.get_ident().unwrap().to_string();

        quote! {
            let Some(mut #target_attr) = world.get_mut::<#target_type>(entity) else {
                warn!(
                    "Failed to set {} property on entity {}: No {} component found!",
                    #target_attr_name,
                    entity,
                    #target_type_name
                );
                return;
            };

            if *#target_attr != self.#target_attr {
                *#target_attr = self.#target_attr;
            }
        }
    } else if let Some(target_tupl) = &style_attribute.target_tupl {
        let component_type = target_tupl.clone();
        let component_name: Vec<String> = target_tupl
            .clone()
            .into_iter()
            .map(|tt| tt.to_string())
            .collect();
        let component_name = component_name.join("");

        quote! {
            let Some(mut #target_attr) = world.get_mut::<#component_type>(entity) else {
                warn!(
                    "Failed to set {} property on entity {}: No {} component found!",
                    #target_attr_name,
                    entity,
                    #component_name,
                );
                return;
            };

            if #target_attr.0 != self.#target_attr {
                #target_attr.0 = self.#target_attr;
            }
        }
    } else if let (Some(target_component), Some(component_attr)) = (
        &style_attribute.target_component,
        &style_attribute.target_component_attr,
    ) {
        let component_type = target_component.clone();
        let component_name: Vec<String> = target_component
            .clone()
            .into_iter()
            .map(|tt| tt.to_string())
            .collect();
        let component_name = component_name.join("");
        let attr_name = component_attr.to_string();

        quote! {
            let Some(mut #target_attr) = world.get_mut::<#component_type>(entity) else {
                warn!(
                    "Failed to set {} property on entity {}: No {} component found!",
                    #attr_name,
                    entity,
                    #component_name,
                );
                return;
            };

            if #target_attr.bypass_change_detection().#component_attr != self.#target_attr {
                #target_attr.#component_attr = self.#target_attr;
            }
        }
    } else if let Some(target_component) = &style_attribute.target_component {
        let component_type = target_component.clone();
        let component_name: Vec<String> = target_component
            .clone()
            .into_iter()
            .map(|tt| tt.to_string())
            .collect();
        let component_name = component_name.join("");

        quote! {
            let Some(mut #target_attr) = world.get_mut::<#component_type>(entity) else {
                warn!(
                    "Failed to set {} property on entity {}: No {} component found!",
                    #target_attr_name,
                    entity,
                    #component_name,
                );
                return;
            };

            if *(#target_attr.bypass_change_detection()) != self.#target_attr {
                world.entity_mut(entity).insert(self.#target_attr);
            }
        }
    } else {
        quote! {
            let Some(mut style) = world.get_mut::<Node>(entity) else {
                warn!(
                    "Failed to set {} property on entity {}: No Node component found!",
                    #target_attr_name,
                    entity
                );
                return;
            };

            if style.#target_attr != self.#target_attr {
                style.#target_attr = self.#target_attr;
            }
        }
    }
}
