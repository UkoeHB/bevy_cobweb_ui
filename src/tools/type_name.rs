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

assert_eq!("MyStruct", MyStruct::NAME);
```
*/
pub trait TypeName
{
    /// The type's name as a constant.
    const NAME: &'static str;
}

//-------------------------------------------------------------------------------------------------------------------
