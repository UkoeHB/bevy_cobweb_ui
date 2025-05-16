use std::sync::Arc;

use bevy_cobweb_ui::cob::*;

use super::helpers::{test_cob, test_cob_fail};

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

    let res = test_cob(
        b"#manifest
",
    );
    let CobSection::Manifest(manifest) = &res.sections[0] else { unreachable!() };
    assert_eq!(manifest.entries.len(), 0);

    let res = test_cob(
        b"#manifest
self as a
",
    );
    let CobSection::Manifest(manifest) = &res.sections[0] else { unreachable!() };
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(manifest.entries[0].file, CobManifestFile::SelfRef);
    assert_eq!(manifest.entries[0].key, ManifestKey(Arc::from("a")));

    let res = test_cob(
        b"
#manifest
self as a.b
\"path/to/b.cob\" as a.b.c
",
    );
    let CobSection::Manifest(manifest) = &res.sections[0] else { unreachable!() };
    assert_eq!(manifest.entries.len(), 2);
    assert_eq!(manifest.entries[0].file, CobManifestFile::SelfRef);
    assert_eq!(manifest.entries[0].key, ManifestKey(Arc::from("a.b")));
    assert_eq!(manifest.entries[1].file, CobManifestFile::File(CobFile::try_new("path/to/b.cob").unwrap()));
    assert_eq!(manifest.entries[1].key, ManifestKey(Arc::from("a.b.c")));
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
    test_cob_fail(
        b"#manifest
self as a
1",
        b"1",
    );
    // File not ending in .cob
    test_cob_fail(
        b"#manifest
\"a.cob.json as a\"",
        b"\"a.cob.json as a\"",
    );
    // Section not starting on newline
    test_cob_fail(
        b" #manifest
",
        b"#manifest\n",
    );
    // Entry not starting with newline
    test_cob_fail(
        b"#manifest
 self as a",
        b"self as a",
    );
    // No fill after 'as'
    test_cob_fail(
        b"#manifest
self asa",
        b"a",
    );
    // No fill before 'as'
    test_cob_fail(
        b"#manifest
selfas a",
        b"as a",
    );
    // Manifest key not lowercase
    test_cob_fail(
        b"#manifest
self as A",
        b"A",
    );
    // Manifest key starting with '.'
    test_cob_fail(
        b"#manifest
self as .a",
        b".a",
    );
    // Manifest key not interspersed with single '.'
    test_cob_fail(
        b"#manifest
self as a..b",
        b"..b",
    );
}

//-------------------------------------------------------------------------------------------------------------------
