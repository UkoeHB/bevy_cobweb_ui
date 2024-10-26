use std::sync::Arc;

use bevy_cobweb_ui::prelude::caf::*;
use smol_str::SmolStr;

use super::helpers::{test_caf, test_caf_fail};

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

    let res = test_caf(
        b"#import
",
    );
    let CafSection::Import(import) = &res.sections[0] else { unreachable!() };
    assert_eq!(import.entries.len(), 0);

    let res = test_caf(
        b"#import
a as _
",
    );
    let CafSection::Import(import) = &res.sections[0] else { unreachable!() };
    assert_eq!(import.entries.len(), 1);
    assert_eq!(import.entries[0].key, CafManifestKey(Arc::from("a")));
    assert_eq!(import.entries[0].alias, CafImportAlias::None);

    let res = test_caf(
        b"
#import
a as a
a.b as a::b
a.b.c as a::b::c
",
    );
    let CafSection::Import(import) = &res.sections[0] else { unreachable!() };
    assert_eq!(import.entries.len(), 3);
    assert_eq!(import.entries[0].key, CafManifestKey(Arc::from("a")));
    assert_eq!(import.entries[0].alias, CafImportAlias::Alias(SmolStr::from("a")));
    assert_eq!(import.entries[1].key, CafManifestKey(Arc::from("a.b")));
    assert_eq!(import.entries[1].alias, CafImportAlias::Alias(SmolStr::from("a::b")));
    assert_eq!(import.entries[2].key, CafManifestKey(Arc::from("a.b.c")));
    assert_eq!(import.entries[2].alias, CafImportAlias::Alias(SmolStr::from("a::b::c")));
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
    test_caf_fail(
        b"#import
a as _
1",
        b"1",
    );
    // Section not starting on newline
    test_caf_fail(
        b" #import
",
        b"#import\n",
    );
    // Entry not starting with newline
    test_caf_fail(
        b"#import
 a as a",
        b"a as a",
    );
    // No fill after 'as'
    test_caf_fail(
        b"#import
a asa",
        b"a",
    );
    // No fill before 'as'
    test_caf_fail(
        b"#import
aas a",
        b"a",
    );
    // Alias not lowercase
    test_caf_fail(
        b"#import
a as A",
        b"A",
    );
    // Alias starting with '::'
    test_caf_fail(
        b"#import
a as ::a",
        b"::a",
    );
    // Alias not interspersed with single '::'
    test_caf_fail(
        b"#import
a.b as a:::b",
        b":::b",
    );
}

//-------------------------------------------------------------------------------------------------------------------
