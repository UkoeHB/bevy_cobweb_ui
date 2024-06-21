//-------------------------------------------------------------------------------------------------------------------

pub(crate) const MANIFEST_KEYWORD: &str = "#manifest";
pub(crate) const IMPORT_KEYWORD: &str = "#import";
pub(crate) const USING_KEYWORD: &str = "#using";
pub(crate) const CONSTANTS_KEYWORD: &str = "#constants";
pub(crate) const COMMANDS_KEYWORD: &str = "#commands";
pub(crate) const COMMENT_KEYWORD: &str = "#c:";

pub(crate) const SPECS_KEYWORD: &str = "#specs";
pub(crate) const SPEC_INVOCATION_KEYWORD: &str = "#spec:";
pub(crate) const SPEC_PARAMETER_MARKER: &str = "@";
pub(crate) const SPEC_INSERTION_MARKER: &str = "!";
pub(crate) const SPEC_CONTENT_SYMBOL: &str = "*";

pub(crate) const CONSTANT_MARKER: &str = "$";
pub(crate) const CONSTANT_IN_CONSTANT_MARKER: &str = "$$";
pub(crate) const CONSTANT_PASTE_ALL_TERMINATOR: &str = "*";
pub(crate) const CONSTANT_SEPARATOR: &str = "::";

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if `key` is a keyword for a section of JSON that cannot be edited by specs.
///
/// Spec-editable sections: specs, commands, base loadables.
pub(crate) fn is_keyword_for_non_spec_editable_section(key: &str) -> bool
{
    key == MANIFEST_KEYWORD
        || key == IMPORT_KEYWORD
        || key == USING_KEYWORD
        || key == CONSTANTS_KEYWORD
        || key.starts_with(COMMENT_KEYWORD)
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if `key` is a keyword for a section of JSON that cannot be edited by constants.
///
/// Constant-editable sections: specs, commands, base loadables.
pub(crate) fn is_keyword_for_non_constant_editable_section(key: &str) -> bool
{
    is_keyword_for_non_spec_editable_section(key)
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if `key` is any keyword.
pub(crate) fn is_any_keyword(key: &str) -> bool
{
    is_keyword_for_non_constant_editable_section(key) || key == SPECS_KEYWORD || key == COMMANDS_KEYWORD
}

//-------------------------------------------------------------------------------------------------------------------
