//-------------------------------------------------------------------------------------------------------------------

/// Trait for converting a type to a string at compile time.
///
/// Use the `TypeName` derive to implement this on your types.
/// The derive only works for non-generic types.
/*
Example:
```rust
#[derive(TypeName)]
struct MyStruct;

assert_eq!("MyStruct", MyStruct::type_name());
```
*/
pub trait TypeName
{
    /// The type name as a constant.
    const TYPE_NAME: &'static str;

    /// Gets the type name.
    fn type_name() -> &'static str;
}

//-------------------------------------------------------------------------------------------------------------------
