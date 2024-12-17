use bevy_cobweb_ui::prelude::cob::*;

use super::helpers::{test_cob, test_cob_fail};

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn commands_section()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let res = test_cob(
        b"#commands
",
    );
    let CobSection::Commands(commands) = &res.sections[0] else { unreachable!() };
    assert_eq!(commands.entries.len(), 0);

    let res = test_cob(
        b"#commands
A
",
    );
    let CobSection::Commands(commands) = &res.sections[0] else { unreachable!() };
    assert_eq!(commands.entries.len(), 1);
    let CobCommandEntry(instruction) = &commands.entries[0];
    assert_eq!(instruction.id.to_canonical(None), "A");

    let res = test_cob(
        b"
#commands
A
B<A>
C<D>::X{ a: 1, b: 2 }
",
    );
    let CobSection::Commands(commands) = &res.sections[0] else { unreachable!() };
    assert_eq!(commands.entries.len(), 3);
    let CobCommandEntry(instruction) = &commands.entries[0];
    assert_eq!(instruction.id.to_canonical(None), "A");
    let CobCommandEntry(instruction) = &commands.entries[1];
    assert_eq!(instruction.id.to_canonical(None), "B<A>");
    let CobCommandEntry(instruction) = &commands.entries[2];
    assert_eq!(instruction.id.to_canonical(None), "C<D>");
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn commands_errors()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // Non-entry in section
    test_cob_fail(
        b"#commands
A
1",
        b"1",
    );
    // Section not starting on newline
    test_cob_fail(
        b" #commands
",
        b"#commands\n",
    );
    // Entry not starting with newline
    test_cob_fail(
        b"#commands
 A",
        b"A",
    );
}

//-------------------------------------------------------------------------------------------------------------------
