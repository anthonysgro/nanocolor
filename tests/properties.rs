use nanocolor::with_colors_override;
use nanocolor::Color;
use proptest::prelude::*;

fn any_color() -> impl Strategy<Value = Color> {
    prop_oneof![
        Just(Color::Black),
        Just(Color::Red),
        Just(Color::Green),
        Just(Color::Yellow),
        Just(Color::Blue),
        Just(Color::Magenta),
        Just(Color::Cyan),
        Just(Color::White),
        Just(Color::BrightBlack),
        Just(Color::BrightRed),
        Just(Color::BrightGreen),
        Just(Color::BrightYellow),
        Just(Color::BrightBlue),
        Just(Color::BrightMagenta),
        Just(Color::BrightCyan),
        Just(Color::BrightWhite),
    ]
}

// Feature: nanocolor, Property 1: SGR code correctness for all colors
// **Validates: Requirements 2.3, 3.3**
proptest! {
    #[test]
    fn prop_sgr_code_correctness_colors(color in any_color()) {
        let fg = color.fg_code();
        let bg = color.bg_code();

        // fg_code must be in 30-37 (standard) or 90-97 (bright)
        let valid_fg = (30..=37).contains(&fg) || (90..=97).contains(&fg);
        prop_assert!(valid_fg, "fg_code {} not in valid range", fg);

        // bg_code must be exactly fg_code + 10
        prop_assert_eq!(bg, fg + 10, "bg_code should be fg_code + 10");
    }
}

use nanocolor::Style;

fn any_style() -> impl Strategy<Value = Style> {
    prop_oneof![
        Just(Style::Bold),
        Just(Style::Dim),
        Just(Style::Italic),
        Just(Style::Underline),
        Just(Style::Blink),
        Just(Style::RapidBlink),
        Just(Style::Reverse),
        Just(Style::Hidden),
        Just(Style::Strikethrough),
        Just(Style::Overline),
    ]
}

// Feature: nanocolor, Property 2: SGR code correctness for all styles
// **Validates: Requirements 4.2**
proptest! {
    #[test]
    fn prop_sgr_code_correctness_styles(style in any_style()) {
        let code = style.code();
        let expected = match style {
            Style::Bold => 1,
            Style::Dim => 2,
            Style::Italic => 3,
            Style::Underline => 4,
            Style::Blink => 5,
            Style::RapidBlink => 6,
            Style::Reverse => 7,
            Style::Hidden => 8,
            Style::Strikethrough => 9,
            Style::Overline => 53,
        };
        prop_assert_eq!(code, expected, "Style {:?} should have code {}, got {}", style, expected, code);
    }
}

use nanocolor::Colorize;

fn any_style_subset() -> impl Strategy<Value = Vec<Style>> {
    prop::collection::hash_set(any_style(), 1..=5)
        .prop_map(|s: std::collections::HashSet<Style>| s.into_iter().collect::<Vec<_>>())
}

// Feature: nanocolor, Property 3: Chaining accumulates all specified codes
// **Validates: Requirements 1.3, 4.3**
proptest! {
    #[test]
    fn prop_chaining_accumulates_all_codes(
        fg in any_color(),
        bg in any_color(),
        styles in any_style_subset(),
    ) {
        let styled = "test".styled();
        // Apply fg
        let styled = match fg {
            Color::Black => styled.black(),
            Color::Red => styled.red(),
            Color::Green => styled.green(),
            Color::Yellow => styled.yellow(),
            Color::Blue => styled.blue(),
            Color::Magenta => styled.magenta(),
            Color::Cyan => styled.cyan(),
            Color::White => styled.white(),
            Color::BrightBlack => styled.bright_black(),
            Color::BrightRed => styled.bright_red(),
            Color::BrightGreen => styled.bright_green(),
            Color::BrightYellow => styled.bright_yellow(),
            Color::BrightBlue => styled.bright_blue(),
            Color::BrightMagenta => styled.bright_magenta(),
            Color::BrightCyan => styled.bright_cyan(),
            Color::BrightWhite => styled.bright_white(),
        };
        // Apply bg
        let styled = match bg {
            Color::Black => styled.on_black(),
            Color::Red => styled.on_red(),
            Color::Green => styled.on_green(),
            Color::Yellow => styled.on_yellow(),
            Color::Blue => styled.on_blue(),
            Color::Magenta => styled.on_magenta(),
            Color::Cyan => styled.on_cyan(),
            Color::White => styled.on_white(),
            Color::BrightBlack => styled.on_bright_black(),
            Color::BrightRed => styled.on_bright_red(),
            Color::BrightGreen => styled.on_bright_green(),
            Color::BrightYellow => styled.on_bright_yellow(),
            Color::BrightBlue => styled.on_bright_blue(),
            Color::BrightMagenta => styled.on_bright_magenta(),
            Color::BrightCyan => styled.on_bright_cyan(),
            Color::BrightWhite => styled.on_bright_white(),
        };
        // Apply styles
        let mut styled = styled;
        for style in &styles {
            styled = match style {
                Style::Bold => styled.bold(),
                Style::Dim => styled.dim(),
                Style::Italic => styled.italic(),
                Style::Underline => styled.underline(),
                Style::Blink => styled.blink(),
                Style::RapidBlink => styled.rapid_blink(),
                Style::Reverse => styled.reverse(),
                Style::Hidden => styled.hidden(),
                Style::Strikethrough => styled.strikethrough(),
                Style::Overline => styled.overline(),
            };
        }

        // Verify fg is set correctly
        prop_assert_eq!(styled.fg, Some(fg));
        // Verify bg is set correctly
        prop_assert_eq!(styled.bg, Some(bg));
        // Verify all styles are accumulated
        for style in &styles {
            prop_assert!(
                styled.styles.contains(style),
                "Style {:?} missing from accumulated styles {:?}",
                style,
                styled.styles
            );
        }
    }
}

fn any_text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _!@#%^&]{1,20}".prop_filter("no ANSI escapes", |s| !s.contains('\x1b'))
}

fn any_style_subset_nonempty() -> impl Strategy<Value = Vec<Style>> {
    prop::collection::hash_set(any_style(), 1..=5).prop_map(|s| s.into_iter().collect::<Vec<_>>())
}

// Feature: nanocolor, Property 4: Display output format correctness
// **Validates: Requirements 1.4, 7.3**
proptest! {
    #[test]
    fn prop_display_output_format_with_styling(
        text in any_text(),
        fg in proptest::option::of(any_color()),
        bg in proptest::option::of(any_color()),
        styles in any_style_subset_nonempty(),
    ) {
        // Use with_colors_override to hold a lock while testing ANSI output format
        let output = with_colors_override(true, || {
            let mut styled = text.clone().styled();
            styled.fg = fg;
            styled.bg = bg;
            styled.styles = styles.clone();
            format!("{}", styled)
        });

        // Build expected codes in the same order as Display: styles, fg, bg
        let mut codes: Vec<String> = Vec::new();
        for s in &styles {
            codes.push(s.code().to_string());
        }
        if let Some(f) = fg {
            codes.push(f.fg_code().to_string());
        }
        if let Some(b) = bg {
            codes.push(b.bg_code().to_string());
        }

        let expected = format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text);
        prop_assert_eq!(output, expected);
    }

    #[test]
    fn prop_display_plain_string_no_escapes(text in any_text()) {
        // A StyledString with no color/style should produce plain text
        let styled = text.clone().styled();
        let output = format!("{}", styled);
        prop_assert_eq!(output, text);
    }
}

use nanocolor::style;

// Feature: nanocolor-v2-features, Property 1: Display type text preservation
// **Validates: Requirements 1.1, 1.2**
proptest! {
    #[test]
    fn prop_display_type_text_preservation_i32(val in proptest::num::i32::ANY) {
        let expected = val.to_string();
        let styled = val.styled();
        prop_assert_eq!(styled.text.as_ref(), expected.as_str());
        let styled_via_fn = style(val);
        prop_assert_eq!(styled_via_fn.text.as_ref(), expected.as_str());
    }

    #[test]
    fn prop_display_type_text_preservation_f64(val in proptest::num::f64::ANY) {
        let expected = val.to_string();
        let styled = val.styled();
        prop_assert_eq!(styled.text.as_ref(), expected.as_str());
        let styled_via_fn = style(val);
        prop_assert_eq!(styled_via_fn.text.as_ref(), expected.as_str());
    }

    #[test]
    fn prop_display_type_text_preservation_bool(val in proptest::bool::ANY) {
        let expected = val.to_string();
        let styled = val.styled();
        prop_assert_eq!(styled.text.as_ref(), expected.as_str());
        let styled_via_fn = style(val);
        prop_assert_eq!(styled_via_fn.text.as_ref(), expected.as_str());
    }

    #[test]
    fn prop_display_type_text_preservation_char(val in proptest::char::any()) {
        let expected = val.to_string();
        let styled = val.styled();
        prop_assert_eq!(styled.text.as_ref(), expected.as_str());
        let styled_via_fn = style(val);
        prop_assert_eq!(styled_via_fn.text.as_ref(), expected.as_str());
    }
}
/// Helper: build a StyledString from text, optional fg, optional bg, and styles.
/// Used to construct two identical StyledStrings for comparison tests.
fn build_styled(
    text: &str,
    fg: Option<Color>,
    bg: Option<Color>,
    styles: &[Style],
) -> nanocolor::StyledString<'static> {
    let mut s = text.to_string().styled();
    s.fg = fg;
    s.bg = bg;
    s.styles = styles.to_vec();
    s
}

// Feature: nanocolor-v2-features, Property 2: .whenever(true) is identity
// **Validates: Requirements 2.2, 2.4**
proptest! {
    #[test]
    fn prop_whenever_true_is_identity(
        text in any_text(),
        fg in proptest::option::of(any_color()),
        bg in proptest::option::of(any_color()),
        styles in any_style_subset(),
    ) {
        // Build two identical StyledStrings: one with .whenever(true), one without
        let with_whenever = build_styled(&text, fg, bg, &styles).whenever(true);
        let without_whenever = build_styled(&text, fg, bg, &styles);

        // Both should render identically regardless of colors_enabled() state,
        // because .whenever(true) defers to colors_enabled() — same as the default.
        // Use with_colors_override to hold a lock and prevent races.
        let (output_with, output_without) = with_colors_override(false, || {
            (format!("{}", with_whenever), format!("{}", without_whenever))
        });

        prop_assert_eq!(
            output_with, output_without,
            "whenever(true) should be identity: output should match no-whenever version"
        );
    }
}

// Feature: nanocolor-v2-features, Property 3: .whenever(false) strips ANSI codes
// **Validates: Requirements 2.3**
proptest! {
    #[test]
    fn prop_whenever_false_strips_ansi(
        text in any_text(),
        fg in proptest::option::of(any_color()),
        bg in proptest::option::of(any_color()),
        styles in any_style_subset(),
    ) {
        // Build a StyledString with arbitrary styling, then apply .whenever(false)
        let styled = build_styled(&text, fg, bg, &styles).whenever(false);

        // .whenever(false) forces styling off, so output must be plain text
        let output = format!("{}", styled);

        prop_assert_eq!(
            output, text,
            "whenever(false) should strip all ANSI codes and return plain text"
        );
    }
}

// Feature: nanocolor-v2-features, Property 4: .whenever() last-write-wins
// **Validates: Requirements 2.5**
proptest! {
    #[test]
    fn prop_whenever_last_write_wins(
        text in any_text(),
        fg in proptest::option::of(any_color()),
        bg in proptest::option::of(any_color()),
        styles in any_style_subset(),
        a in proptest::bool::ANY,
        b in proptest::bool::ANY,
    ) {
        // Build one StyledString with .whenever(a).whenever(b) — two calls
        let chained = build_styled(&text, fg, bg, &styles).whenever(a).whenever(b);

        // Build another identical StyledString with only .whenever(b) — single call
        let single = build_styled(&text, fg, bg, &styles).whenever(b);

        // The last call should win, so both must render identically.
        // Use with_colors_override to hold a lock and prevent races.
        let (output_chained, output_single) = with_colors_override(false, || {
            (format!("{}", chained), format!("{}", single))
        });

        prop_assert_eq!(
            output_chained, output_single,
            "whenever(a).whenever(b) should equal whenever(b): last-write-wins"
        );
    }
}

// Feature: nanocolor-v2-features, Property 5: Mask with active styling renders normally
// **Validates: Requirements 3.2**
proptest! {
    #[test]
    fn prop_mask_with_active_styling_renders_normally(
        text in any_text(),
        fg in proptest::option::of(any_color()),
        bg in proptest::option::of(any_color()),
        styles in any_style_subset(),
    ) {
        // Use with_colors_override to hold a lock while testing mask behavior
        let (output_masked, output_unmasked) = with_colors_override(true, || {
            let masked = build_styled(&text, fg, bg, &styles).mask();
            let unmasked = build_styled(&text, fg, bg, &styles);
            (format!("{}", masked), format!("{}", unmasked))
        });

        // When styling is active, .mask() should have no effect on output
        prop_assert_eq!(
            output_masked, output_unmasked,
            "mask() with active styling should render identically to no mask"
        );
    }
}

// Feature: nanocolor-v2-features, Property 6: Mask with inactive styling renders empty
// **Validates: Requirements 3.3, 3.4, 3.5**
proptest! {
    #[test]
    fn prop_mask_colors_disabled_renders_empty(
        text in any_text(),
        fg in proptest::option::of(any_color()),
        bg in proptest::option::of(any_color()),
        styles in any_style_subset(),
    ) {
        // Use .whenever(false) to force styling inactive per-value,
        // avoiding global state races with parallel test threads
        let masked = build_styled(&text, fg, bg, &styles).mask().whenever(false);
        let output = format!("{}", masked);

        prop_assert_eq!(
            output, "",
            "mask() with styling inactive should render empty string"
        );
    }

    #[test]
    fn prop_mask_whenever_false_renders_empty(
        text in any_text(),
        fg in proptest::option::of(any_color()),
        bg in proptest::option::of(any_color()),
        styles in any_style_subset(),
        colors_on in proptest::bool::ANY,
    ) {
        // .whenever(false) override: masked value should render empty regardless of colors_enabled
        let output = with_colors_override(colors_on, || {
            let masked = build_styled(&text, fg, bg, &styles).mask().whenever(false);
            format!("{}", masked)
        });

        prop_assert_eq!(
            output, "",
            "mask().whenever(false) should render empty string regardless of colors_enabled"
        );
    }
}
