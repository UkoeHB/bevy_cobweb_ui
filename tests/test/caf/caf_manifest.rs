use std::sync::Arc;

use bevy_cobweb_ui::prelude::*;

use super::helpers::{test_caf, test_caf_fail};

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn manifest_section()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let res = test_caf(
        b"#manifest
",
    );
    let CafSection::Manifest(manifest) = &res.sections[0] else { unreachable!() };
    assert_eq!(manifest.entries.len(), 0);

    let res = test_caf(
        b"#manifest
self as a
",
    );
    let CafSection::Manifest(manifest) = &res.sections[0] else { unreachable!() };
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(manifest.entries[0].file, CafManifestFile::SelfRef);
    assert_eq!(manifest.entries[0].key, CafManifestKey(Arc::from("a")));

    let res = test_caf(
        b"
#manifest
self as a.b
\"path/to/b.caf\" as a.b.c
",
    );
    let CafSection::Manifest(manifest) = &res.sections[0] else { unreachable!() };
    assert_eq!(manifest.entries.len(), 2);
    assert_eq!(manifest.entries[0].file, CafManifestFile::SelfRef);
    assert_eq!(manifest.entries[0].key, CafManifestKey(Arc::from("a.b")));
    assert_eq!(manifest.entries[1].file, CafManifestFile::File(CafFilePath(Arc::from("path/to/b.caf"))));
    assert_eq!(manifest.entries[1].key, CafManifestKey(Arc::from("a.b.c")));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn manifest_errors()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // Non-entry in section
    test_caf_fail(
        b"#manifest
self as a
1",
        b"1",
    );
    // File not ending in .caf
    test_caf_fail(
        b"#manifest
\"a.caf.json as a\"",
        b"\"a.caf.json as a\"",
    );
    // Entry not starting with newline
    test_caf_fail(
        b"#manifest
 self as a",
        b"self as a",
    );
    // No fill after 'as'
    test_caf_fail(
        b"#manifest
self asa",
        b"a",
    );
    // No fill before 'as'
    test_caf_fail(
        b"#manifest
selfas a",
        b"as a",
    );
    // Manifest key not lowercase
    test_caf_fail(
        b"#manifest
self as A",
        b"A",
    );
    // Manifest key starting with '.'
    test_caf_fail(
        b"#manifest
self as .a",
        b".a",
    );
    // Manifest key not interspersed with single '.'
    test_caf_fail(
        b"#manifest
self as a..b",
        b"..b",
    );
}

//-------------------------------------------------------------------------------------------------------------------
