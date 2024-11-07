use std::cell::Cell;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

thread_local! {
    static RECURSION_LIMIT: Cell<u32> = Cell::new(250);
    static RECURSION_COUNT: Cell<u32> = Cell::new(0);
}

//-------------------------------------------------------------------------------------------------------------------

fn get_local_recursion_limit() -> u32
{
    RECURSION_LIMIT.get()
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn get_local_recursion_count() -> u32
{
    RECURSION_COUNT.get()
}

//-------------------------------------------------------------------------------------------------------------------

fn _set_local_recursion_limit(limit: u32)
{
    RECURSION_LIMIT.set(limit);
}

//-------------------------------------------------------------------------------------------------------------------

fn set_local_recursion_count(count: u32)
{
    RECURSION_COUNT.set(count);
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns false if the count becomes >= the limit.
fn try_increment_recursion_count() -> bool
{
    let limit = get_local_recursion_limit();
    let count = get_local_recursion_count();

    if count + 1 >= limit {
        return false;
    }

    set_local_recursion_count(count + 1);

    true
}

//-------------------------------------------------------------------------------------------------------------------

fn decrement_recursion_count()
{
    let count = get_local_recursion_count();
    set_local_recursion_count(count.saturating_sub(1));
}

//-------------------------------------------------------------------------------------------------------------------

/// Recursion-count a parser callback.
///
/// TODO: Calling this increases the stack by 2 layers. One for `rc()`, and another for the callback.
pub(crate) fn rc<'a, T>(
    content: Span<'a>,
    callback: impl FnOnce(Span<'a>) -> Result<T, SpanError<'a>>,
) -> Result<T, SpanError<'a>>
{
    if !try_increment_recursion_count() {
        tracing::warn!("aborting CAF parse at {:?}; exceeded recursion limit of {}",
            get_location(content).as_str(), get_local_recursion_limit());
        return Err(span_verify_failure(content));
    }
    let res = (callback)(content);
    decrement_recursion_count();
    res
}

//-------------------------------------------------------------------------------------------------------------------
