//! The player's journal, which SPD keeps in the static `Document` state.
//!
//! Level generation reads it in three places: the random Adventurer's Guide
//! page (`RegularLevel.java:559-583`), `LaboratoryRoom`'s Alchemy Guide pages
//! (`LaboratoryRoom.java:86-114`), and floor 2's entrance guide page
//! (`EntranceRoom.java:119-131`). Region lore pages drop only once the guide
//! is complete and their region's document is not, so both profiles here
//! skip them.
//!
//! The default assumes every document is complete, as on a veteran profile:
//! no page drops and no page roll consumes RNG. A fresh journal (every page
//! missing apart from the read intro) matches the Java oracle fixtures.

use std::cell::Cell;

thread_local! {
    static LORE_COMPLETE: Cell<bool> = const { Cell::new(true) };
}

/// True when every journal document counts as found.
pub fn lore_complete() -> bool {
    LORE_COMPLETE.with(Cell::get)
}

/// Sets the journal state for later generation on this thread.
pub fn set_lore_complete(complete: bool) {
    LORE_COMPLETE.with(|state| state.set(complete));
}

/// Restores the previous journal state when dropped.
#[must_use = "the journal state reverts when the scope is dropped"]
pub struct LoreScope(bool);

impl Drop for LoreScope {
    fn drop(&mut self) {
        set_lore_complete(self.0);
    }
}

/// Sets the journal state until the returned scope is dropped.
pub fn scope_lore_complete(complete: bool) -> LoreScope {
    let scope = LoreScope(lore_complete());
    set_lore_complete(complete);
    scope
}

/// Runs `f` with the given journal state, then restores the previous one.
pub fn with_lore_complete<T>(complete: bool, f: impl FnOnce() -> T) -> T {
    let _scope = scope_lore_complete(complete);
    f()
}
