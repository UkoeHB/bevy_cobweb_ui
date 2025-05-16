use smol_str::SmolStr;

//-------------------------------------------------------------------------------------------------------------------

pub(super) const DEFS_SEPARATOR: &str = "::";

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn path_to_string<T: AsRef<str>>(separator: &str, path: &[T]) -> SmolStr
{
    // skip empties and concatenate: a::b::c
    let mut count = 0;
    SmolStr::from_iter(
        path.iter()
            .filter(|p| !p.as_ref().is_empty())
            .flat_map(|p| {
                count += 1;
                match count {
                    1 => ["", p.as_ref()],
                    _ => [separator, p.as_ref()],
                }
            }),
    )
}

//-------------------------------------------------------------------------------------------------------------------
