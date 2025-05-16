use std::sync::Arc;

use bevy_cobweb_ui::cob::*;
use smol_str::SmolStr;

use super::helpers::{test_cob, test_cob_fail};

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn import_section()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let res = test_cob(
        b"#import
",
    );
    let CobSection::Import(import) = &res.sections[0] else { unreachable!() };
    assert_eq!(import.entries.len(), 0);

    let res = test_cob(
        b"#import
a as _
",
    );
    let CobSection::Import(import) = &res.sections[0] else { unreachable!() };
    assert_eq!(import.entries.len(), 1);
    assert_eq!(import.entries[0].key, ManifestKey(Arc::from("a")));
    assert_eq!(import.entries[0].alias, CobImportAlias::None);

    let res = test_cob(
        b"
#import
a as a
a.b as a::b
a.b.c as a::b::c
",
    );
    let CobSection::Import(import) = &res.sections[0] else { unreachable!() };
    assert_eq!(import.entries.len(), 3);
    assert_eq!(import.entries[0].key, ManifestKey(Arc::from("a")));
    assert_eq!(import.entries[0].alias, CobImportAlias::Alias(SmolStr::from("a")));
    assert_eq!(import.entries[1].key, ManifestKey(Arc::from("a.b")));
    assert_eq!(import.entries[1].alias, CobImportAlias::Alias(SmolStr::from("a::b")));
    assert_eq!(import.entries[2].key, ManifestKey(Arc::from("a.b.c")));
    assert_eq!(import.entries[2].alias, CobImportAlias::Alias(SmolStr::from("a::b::c")));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn import_errors()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // Non-entry in section
    test_cob_fail(
        b"#import
a as _
1",
        b"1",
    );
    // Section not starting on newline
    test_cob_fail(
        b" #import
",
        b"#import\n",
    );
    // Entry not starting with newline
    test_cob_fail(
        b"#import
 a as a",
        b"a as a",
    );
    // No fill after 'as'
    test_cob_fail(
        b"#import
a asa",
        b"a",
    );
    // No fill before 'as'
    test_cob_fail(
        b"#import
aas a",
        b"a",
    );
    // Alias not lowercase
    test_cob_fail(
        b"#import
a as A",
        b"A",
    );
    // Alias starting with '::'
    test_cob_fail(
        b"#import
a as ::a",
        b"::a",
    );
    // Alias not interspersed with single '::'
    test_cob_fail(
        b"#import
a.b as a:::b",
        b":::b",
    );
}

//-------------------------------------------------------------------------------------------------------------------
