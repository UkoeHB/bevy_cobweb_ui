mod inner;

use proc_macro::TokenStream;

//-------------------------------------------------------------------------------------------------------------------

#[proc_macro_derive(TypeName)]
pub fn derive_type_name(input: TokenStream) -> TokenStream
{
    inner::derive_type_name_impl(input)
}

//-------------------------------------------------------------------------------------------------------------------
