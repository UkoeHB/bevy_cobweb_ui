use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_type_name_impl(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
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
