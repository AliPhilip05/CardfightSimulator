//! Shared interpolation for card motion; geometry comes from the field and hand layouts.
use macroquad::prelude::*;

pub(super) fn smoothstep(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub(super) fn draw_progress(phase_progress: f32) -> f32 {
    smoothstep((phase_progress - 0.15) / 0.65)
}
pub(super) fn interpolate_rect(from: Rect, to: Rect, t: f32) -> Rect {
    let t = t.clamp(0.0, 1.0);
    Rect::new(
        from.x + (to.x - from.x) * t,
        from.y + (to.y - from.y) * t,
        from.w + (to.w - from.w) * t,
        from.h + (to.h - from.h) * t,
    )
}
