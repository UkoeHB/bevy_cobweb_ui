use bevy_cobweb_ui::prelude::caf::*;

use super::helpers::{test_caf, test_caf_fail};

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

    let res = test_caf(
        b"#commands
",
    );
    let CafSection::Commands(commands) = &res.sections[0] else { unreachable!() };
    assert_eq!(commands.entries.len(), 0);

    let res = test_caf(
        b"#commands
A
",
    );
    let CafSection::Commands(commands) = &res.sections[0] else { unreachable!() };
    assert_eq!(commands.entries.len(), 1);
    let CafCommandEntry::Instruction(instruction) = &commands.entries[0] else { unreachable!() };
    assert_eq!(instruction.id.to_canonical(None), "A");

    let res = test_caf(
        b"
#commands
A
B<A>
C<D>::X{ a: 1, b: 2 }
",
    );
    let CafSection::Commands(commands) = &res.sections[0] else { unreachable!() };
    assert_eq!(commands.entries.len(), 3);
    let CafCommandEntry::Instruction(instruction) = &commands.entries[0] else { unreachable!() };
    assert_eq!(instruction.id.to_canonical(None), "A");
    let CafCommandEntry::Instruction(instruction) = &commands.entries[1] else { unreachable!() };
    assert_eq!(instruction.id.to_canonical(None), "B<A>");
    let CafCommandEntry::Instruction(instruction) = &commands.entries[2] else { unreachable!() };
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
    test_caf_fail(
        b"#commands
A
1",
        b"1",
    );
    // Section not starting on newline
    test_caf_fail(
        b" #commands
",
        b"#commands\n",
    );
    // Entry not starting with newline
    test_caf_fail(
        b"#commands
 A",
        b"A",
    );
}

//-------------------------------------------------------------------------------------------------------------------
