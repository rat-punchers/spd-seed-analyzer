use super::floor_twenty_three::assert_halls_paint_trace;
use spd_core::rooms::init_rooms::BuilderKind;

#[test]
fn aaa_floor_twenty_four_halls_paint_trace_matches_preserved_run() {
    let _journal = crate::fresh_journal();
    assert_halls_paint_trace(
        "AAA-AAA-AAA",
        "aaa-aaa-aaa-floor-24-halls-paint.json",
        24,
        2,
        18,
        Some(BuilderKind::Loop),
    );
}

#[test]
fn abc_floor_twenty_four_halls_paint_trace_matches_preserved_run() {
    let _journal = crate::fresh_journal();
    assert_halls_paint_trace(
        "ABC-DEF-GHI",
        "abc-def-ghi-floor-24-halls-paint.json",
        24,
        2,
        17,
        Some(BuilderKind::Loop),
    );
}

#[test]
fn gfx_floor_twenty_four_halls_paint_trace_matches_loop_builder_history() {
    let _journal = crate::fresh_journal();
    assert_halls_paint_trace(
        "GFX-PZH-DCH",
        "gfx-pzh-dch-floor-24-halls-paint.json",
        24,
        1,
        21,
        Some(BuilderKind::Loop),
    );
}

#[test]
fn afu_floor_twenty_four_halls_paint_trace_matches_retry_history() {
    let _journal = crate::fresh_journal();
    assert_halls_paint_trace(
        "AAA-AAA-AFU",
        "aaa-aaa-afu-floor-24-halls-paint.json",
        24,
        1,
        17,
        Some(BuilderKind::FigureEight),
    );
}
