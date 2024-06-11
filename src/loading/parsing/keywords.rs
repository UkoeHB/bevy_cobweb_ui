//-------------------------------------------------------------------------------------------------------------------

pub(crate) const MANIFEST_KEYWORD: &str = "#manifest";
pub(crate) const IMPORT_KEYWORD: &str = "#import";
pub(crate) const USING_KEYWORD: &str = "#using";
pub(crate) const CONSTANTS_KEYWORD: &str = "#constants";
pub(crate) const COMMENT_KEYWORD: &str = "#c:";

pub(crate) const INHERITED_KEYWORD: &str = "#inherited";

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn key_is_keyword(key: &str) -> bool
{
    // Does not include 'inherited', which apears in map values, not map keys.
    key == MANIFEST_KEYWORD
        || key == IMPORT_KEYWORD
        || key == USING_KEYWORD
        || key == CONSTANTS_KEYWORD
        || key.starts_with(COMMENT_KEYWORD)
}

//-------------------------------------------------------------------------------------------------------------------
