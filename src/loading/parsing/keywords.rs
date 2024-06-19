//-------------------------------------------------------------------------------------------------------------------

pub(crate) const MANIFEST_KEYWORD: &str = "#manifest";
pub(crate) const IMPORT_KEYWORD: &str = "#import";
pub(crate) const USING_KEYWORD: &str = "#using";
pub(crate) const CONSTANTS_KEYWORD: &str = "#constants";
pub(crate) const COMMANDS_KEYWORD: &str = "#commands";
pub(crate) const COMMENT_KEYWORD: &str = "#c:";

//pub(crate) const SPECS_KEYWORD: &str = "#specs";
//pub(crate) const SPEC_INVOCATION_KEYWORD: &str = "#spec:";
//pub(crate) const SPEC_PARAMETER_MARKER: &str = "@";

pub(crate) const CONSTANT_MARKER: &str = "$";
pub(crate) const CONSTANT_IN_CONSTANT_MARKER: &str = "$$";
pub(crate) const CONSTANT_PASTE_ALL_TERMINATOR: &str = "*";

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn key_is_keyword(key: &str) -> bool
{
    key == COMMANDS_KEYWORD || key_is_non_content_keyword(key)
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if `key` is a keyword for a section of JSON that contains no loadable content.
pub(crate) fn key_is_non_content_keyword(key: &str) -> bool
{
    key == MANIFEST_KEYWORD
        || key == IMPORT_KEYWORD
        || key == USING_KEYWORD
        || key == CONSTANTS_KEYWORD
        || key.starts_with(COMMENT_KEYWORD)
}

//-------------------------------------------------------------------------------------------------------------------
