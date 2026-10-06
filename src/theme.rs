//! The DPF look.
//!
//! The app used stock `egui::Visuals::dark()`, which is why it read as bland: every widget was the
//! framework's default grey, and nothing on screen came from the brand. This module defines the
//! real theme — the colours are sampled from the DPF mark itself, so the app and the logo agree.
//!
//! Two rules kept in mind throughout:
//!   * **Contrast first.** A pretty theme that makes body text hard to read is a regression, so the
//!     text colours are chosen for legibility against their own surface, not for looks in isolation.
//!   * **No colour-only meaning.** A locked row is dimmed AND marked; a warning is amber AND says
//!     what is wrong. Colour reinforces, it never carries the message alone.

use egui::{Color32, Rounding, Stroke, Visuals};

// ── Brand palette, sampled from the logo ────────────────────────────────────────────────────────
/// Deepest surface — panel backgrounds.
pub const INK: Color32 = Color32::from_rgb(0x10, 0x22, 0x33);
/// Raised surface — cards, groups, the sidebar.
pub const SURFACE: Color32 = Color32::from_rgb(0x1A, 0x2E, 0x42);
/// Hover / subtle fill.
pub const SURFACE_HI: Color32 = Color32::from_rgb(0x24, 0x3E, 0x56);
/// The brand teal — selection, active state, primary accent.
pub const TEAL: Color32 = Color32::from_rgb(0x15, 0x98, 0x9E);
/// Brighter teal for hover on the accent.
pub const TEAL_HI: Color32 = Color32::from_rgb(0x2A, 0xB6, 0xBC);
/// The brand amber — emphasis, upgrade prompts, "pay attention".
pub const AMBER: Color32 = Color32::from_rgb(0xF2, 0xA3, 0x37);
/// Deep blue from the mark — borders and secondary accents.
pub const BLUE: Color32 = Color32::from_rgb(0x07, 0x65, 0x8C);
/// Primary text.
pub const TEXT: Color32 = Color32::from_rgb(0xE8, 0xF1, 0xF4);
/// Secondary text — hints, captions.
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x9F, 0xB6, 0xC2);
/// Muted — disabled, locked.
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x6B, 0x83, 0x92);
pub const OK: Color32 = Color32::from_rgb(0x5F, 0xC9, 0x8A);
pub const BAD: Color32 = Color32::from_rgb(0xE0, 0x6C, 0x6C);

/// The brand look for the night theme.
pub fn dark() -> Visuals {
    let mut v = Visuals::dark();

    v.panel_fill = INK;
    v.window_fill = SURFACE;
    v.extreme_bg_color = Color32::from_rgb(0x0B, 0x18, 0x24);
    v.faint_bg_color = SURFACE;
    v.window_stroke = Stroke::new(1.0, BLUE.gamma_multiply(0.45));
    v.window_rounding = Rounding::same(10.0);
    v.menu_rounding = Rounding::same(8.0);

    // Widgets
    v.widgets.noninteractive.bg_fill = SURFACE;
    v.widgets.noninteractive.weak_bg_fill = SURFACE;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BLUE.gamma_multiply(0.35));
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    v.widgets.noninteractive.rounding = Rounding::same(8.0);

    v.widgets.inactive.bg_fill = SURFACE_HI.gamma_multiply(0.55);
    v.widgets.inactive.weak_bg_fill = SURFACE_HI.gamma_multiply(0.45);
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.rounding = Rounding::same(8.0);

    // Hover and press take the brand teal, so interaction always reads as "the app responding".
    v.widgets.hovered.bg_fill = TEAL.gamma_multiply(0.55);
    v.widgets.hovered.weak_bg_fill = TEAL.gamma_multiply(0.45);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, TEAL_HI.gamma_multiply(0.8));
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    v.widgets.hovered.rounding = Rounding::same(8.0);

    v.widgets.active.bg_fill = TEAL;
    v.widgets.active.weak_bg_fill = TEAL;
    v.widgets.active.bg_stroke = Stroke::new(1.0, TEAL_HI);
    v.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    v.widgets.active.rounding = Rounding::same(8.0);

    v.widgets.open.bg_fill = SURFACE_HI;
    v.widgets.open.weak_bg_fill = SURFACE_HI;
    v.widgets.open.rounding = Rounding::same(8.0);

    v.selection.bg_fill = TEAL.gamma_multiply(0.55);
    v.selection.stroke = Stroke::new(1.5, TEAL_HI);
    v.hyperlink_color = TEAL_HI;
    v.warn_fg_color = AMBER;
    v.error_fg_color = BAD;

    v
}

/// The brand look for the daylight theme. Not an inversion: the teal and amber are re-tuned so they
/// stay readable on a light surface, and body text is near-black rather than pure black.
pub fn light() -> Visuals {
    let mut v = Visuals::light();

    let paper = Color32::from_rgb(0xF7, 0xFA, 0xFB);
    let card = Color32::from_rgb(0xFF, 0xFF, 0xFF);
    let teal_ink = Color32::from_rgb(0x0E, 0x6E, 0x73); // darker teal: readable on white
    let amber_ink = Color32::from_rgb(0xB0, 0x6E, 0x0E);

    v.panel_fill = paper;
    v.window_fill = card;
    v.extreme_bg_color = Color32::WHITE;
    v.faint_bg_color = Color32::from_rgb(0xEE, 0xF3, 0xF5);
    v.window_stroke = Stroke::new(1.0, Color32::from_rgb(0xD3, 0xDF, 0xE4));
    v.window_rounding = Rounding::same(10.0);
    v.menu_rounding = Rounding::same(8.0);

    let ink = Color32::from_rgb(0x12, 0x1C, 0x24);
    v.widgets.noninteractive.bg_fill = card;
    v.widgets.noninteractive.weak_bg_fill = card;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0xD3, 0xDF, 0xE4));
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, ink);
    v.widgets.noninteractive.rounding = Rounding::same(8.0);

    v.widgets.inactive.bg_fill = Color32::from_rgb(0xEB, 0xF1, 0xF3);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xEB, 0xF1, 0xF3);
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, ink);
    v.widgets.inactive.rounding = Rounding::same(8.0);

    v.widgets.hovered.bg_fill = TEAL.gamma_multiply(0.35);
    v.widgets.hovered.weak_bg_fill = TEAL.gamma_multiply(0.28);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, teal_ink);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, ink);
    v.widgets.hovered.rounding = Rounding::same(8.0);

    v.widgets.active.bg_fill = teal_ink;
    v.widgets.active.weak_bg_fill = teal_ink;
    v.widgets.active.bg_stroke = Stroke::new(1.0, teal_ink);
    v.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    v.widgets.active.rounding = Rounding::same(8.0);

    v.widgets.open.bg_fill = Color32::from_rgb(0xE2, 0xEC, 0xEF);
    v.widgets.open.weak_bg_fill = Color32::from_rgb(0xE2, 0xEC, 0xEF);
    v.widgets.open.rounding = Rounding::same(8.0);

    v.selection.bg_fill = TEAL.gamma_multiply(0.38);
    v.selection.stroke = Stroke::new(1.5, teal_ink);
    v.hyperlink_color = teal_ink;
    v.warn_fg_color = amber_ink;
    v.error_fg_color = Color32::from_rgb(0xB0, 0x2E, 0x2E);

    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Relative luminance, for a contrast check.
    fn lum(c: Color32) -> f32 {
        let f = |v: u8| {
            let s = v as f32 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (l1, l2) = (lum(a), lum(b));
        let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Body text must be comfortably readable on the surface it actually sits on. A theme that
    /// looks nice but fails this is a regression, so it is asserted rather than eyeballed.
    #[test]
    fn body_text_is_readable_on_both_themes() {
        let d = dark();
        assert!(
            contrast(TEXT, d.panel_fill) >= 7.0,
            "dark theme body text fails WCAG AAA on the panel (contrast {:.1})",
            contrast(TEXT, d.panel_fill)
        );
        assert!(
            contrast(TEXT, d.window_fill) >= 7.0,
            "dark theme body text fails AAA on cards (contrast {:.1})",
            contrast(TEXT, d.window_fill)
        );
        // Secondary text is allowed to be softer, but still clear of WCAG AA for body copy.
        assert!(
            contrast(TEXT_DIM, d.panel_fill) >= 4.5,
            "dark theme hint text is too faint (contrast {:.1})",
            contrast(TEXT_DIM, d.panel_fill)
        );

        let l = light();
        assert!(
            contrast(l.widgets.active.fg_stroke.color, l.widgets.active.bg_fill) >= 4.5,
            "light theme text on the active (teal) fill is unreadable"
        );
        assert!(
            contrast(Color32::from_rgb(0x12, 0x1C, 0x24), l.panel_fill) >= 7.0,
            "light theme body text fails AAA"
        );
    }

    /// The brand colours must actually appear, or the theme is not done — this is the guard against
    /// a future edit quietly reverting the app to stock egui grey.
    #[test]
    fn the_theme_uses_the_brand_colours() {
        let d = dark();
        assert_eq!(d.widgets.active.bg_fill, TEAL, "the active state must be brand teal");
        assert_eq!(d.panel_fill, INK, "the panel fill must be the brand ink");
        assert_eq!(d.warn_fg_color, AMBER, "warnings must be brand amber");
        assert_ne!(
            d.widgets.hovered.bg_fill,
            Visuals::dark().widgets.hovered.bg_fill,
            "hover must differ from stock egui, not merely be configured"
        );
    }
}
