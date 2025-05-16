use bevy_cobweb_ui::cob::*;

use super::helpers::*;

//-------------------------------------------------------------------------------------------------------------------

fn test_fill(val: &str, expected: &str)
{
    // Parse CobFill from value.
    let (parsed, _) = CobFill::parse(test_span(val));
    assert_eq!(parsed, CobFill::new(expected));

    // Parse Cob from value.
    if let Ok(parsed_cob) = Cob::parse(test_span(val)) {
        assert_eq!(parsed_cob.end_fill, parsed);
    }

    // Write as raw.
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(reconstructed_raw, expected);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn end_helpers()
{
    assert_eq!(CobFill::new("\n").ends_with_newline(), true);
    assert_eq!(CobFill::new(" \n").ends_with_newline(), true);
    assert_eq!(CobFill::new("\n ").ends_with_newline(), false);
    assert_eq!(CobFill::new(" \n").ends_newline_then_num_spaces(), Some(0));
    assert_eq!(CobFill::new("\n ").ends_newline_then_num_spaces(), Some(1));
    assert_eq!(CobFill::new(" ").ends_newline_then_num_spaces(), None);
    assert_eq!(CobFill::new("\n //").ends_newline_then_num_spaces(), None);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn whitespace()
{
    // Basic
    test_fill("", "");
    test_fill(" ", " ");
    test_fill("  ", "  ");
    test_fill("\n", "\n");
    test_fill("\n\n", "\n\n");
    test_fill(" \n", " \n");
    test_fill("\n ", "\n ");

    // Terminated
    test_fill("a", "");
    test_fill(" a", " ");
    test_fill(" a ", " ");
    test_fill("a ", "");
    test_fill("\na", "\n");
    test_fill("\na\n", "\n");

    // Filler characters
    test_fill("\r", "\r");
    test_fill("\r ", "\r ");
    test_fill(" \r ", " \r ");
    test_fill("\n\r ", "\n\r ");
    test_fill(";,\r", ";,\r");

    // Banned characters
    test_fill("\t", "");
    test_fill("^", "");
    test_fill("ß", "");
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn comments()
{
    // Basic
    test_fill("//", "//");
    test_fill("// ", "// ");
    test_fill(" // ", " // ");
    test_fill("//\n", "//\n");
    test_fill("// \n", "// \n");
    test_fill(" // \n ", " // \n ");
    test_fill("//a\n", "//a\n");
    test_fill("/**/", "/**/");
    test_fill(" /**/ ", " /**/ ");
    test_fill(" /**/ //", " /**/ //");
    test_fill(" /* a */ //\n //\n/* a */", " /* a */ //\n //\n/* a */");
    test_fill(" /*  a", " /*  a");

    // Terminated
    test_fill("//\na ", "//\n");
    test_fill("/**/ a ", "/**/ ");
}

//-------------------------------------------------------------------------------------------------------------------
