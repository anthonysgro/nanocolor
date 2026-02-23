//! Minimal, zero-dependency terminal color and text styling for Rust.
//!
//! nanocolor gives you ANSI 16-color support (foreground and background), common
//! text styles, conditional styling, and decorative masking — all through a
//! chainable trait-based API. Zero dependencies, single file, under 400 lines.
//!
//! # Quick Start
//!
//! Import [`Colorize`] and call color/style methods on strings, numbers, or booleans:
//!
//! ```rust
//! use nanocolor::Colorize;
//!
//! println!("{}", "error".red().bold());
//! println!("{}", "warning".yellow().on_black());
//! println!("{}", 42.cyan());
//! println!("{}", true.green().italic());
//! ```
//!
//! # Styling Any Type
//!
//! [`Colorize`] is implemented for `&str`, `String`, and all common primitives
//! (`i8`–`i128`, `u8`–`u128`, `f32`, `f64`, `bool`, `char`). For custom
//! `Display` types, use the [`style()`] helper:
//!
//! ```rust
//! use nanocolor::{Colorize, style};
//!
//! // Primitives work directly
//! let count = 99.red().bold();
//! let pi = 3.14_f64.green();
//!
//! // Custom types use style()
//! let msg = style(format!("v{}.{}", 2, 0)).cyan();
//! ```
//!
//! # Conditional Styling
//!
//! Use [`.whenever()`](StyledString::whenever) to enable or disable styling
//! per-value, independent of the global color state:
//!
//! ```rust
//! use nanocolor::Colorize;
//!
//! let is_important = true;
//! println!("{}", "alert".red().bold().whenever(is_important));
//!
//! // .whenever(false) always strips ANSI codes
//! println!("{}", "plain".red().whenever(false)); // prints "plain"
//! ```
//!
//! When `.whenever()` is not called, styling follows the global
//! [`colors_enabled()`] state. If called multiple times, the last call wins.
//!
//! # Decorative Masking
//!
//! Use [`.mask()`](StyledString::mask) to mark decorative values (emoji, symbols)
//! that should disappear entirely when styling is inactive:
//!
//! ```rust
//! use nanocolor::Colorize;
//!
//! // When colors are on:  "✓ passed"
//! // When colors are off: " passed"
//! print!("{}", "✓ ".green().mask());
//! println!("passed");
//! ```
//!
//! Masked values render normally when styling is active, and as an empty string
//! when styling is inactive (colors disabled or `.whenever(false)`).
//!
//! # Automatic Color Detection
//!
//! nanocolor automatically detects whether stdout is a TTY. When output is piped
//! to a file or another program, ANSI codes are skipped and plain text is emitted.
//!
//! The [`NO_COLOR`](https://no-color.org) environment variable is also respected —
//! set it to any non-empty value to disable colors globally.
//!
//! # Global Enable / Disable
//!
//! Use [`enable()`] and [`disable()`] to override auto-detection from code:
//!
//! ```rust
//! // CLI app with a --no-color flag
//! nanocolor::disable();
//! ```
//!
//! This is useful for CLI apps that parse their own flags. Call it once early
//! in `main()` and all styled output respects it automatically.
//!
//! # Available Colors
//!
//! | Standard | Bright |
//! |----------|--------|
//! | `black()` | `bright_black()` |
//! | `red()` | `bright_red()` |
//! | `green()` | `bright_green()` |
//! | `yellow()` | `bright_yellow()` |
//! | `blue()` | `bright_blue()` |
//! | `magenta()` | `bright_magenta()` |
//! | `cyan()` | `bright_cyan()` |
//! | `white()` | `bright_white()` |
//!
//! All colors have background variants with the `on_` prefix (e.g. `on_red()`,
//! `on_bright_cyan()`).
//!
//! # Available Styles
//!
//! `bold()`, `dim()`, `italic()`, `underline()`, `blink()`, `rapid_blink()`,
//! `reverse()`, `hidden()`, `strikethrough()`, `overline()`

use std::borrow::Cow;
use std::sync::atomic::{AtomicU8, Ordering as AtomicOrdering};
use std::sync::OnceLock;

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

// Public override: 0 = no override (use auto-detection), 1 = force on, 2 = force off
static COLOR_MODE: AtomicU8 = AtomicU8::new(0);

/// Globally enable ANSI color output, overriding auto-detection.
///
/// Call this early in `main()` to force colors on regardless of TTY detection
/// or the `NO_COLOR` environment variable.
///
/// # Examples
///
/// ```
/// nanocolor::enable();
/// ```
pub fn enable() {
    COLOR_MODE.store(1, AtomicOrdering::SeqCst);
}

/// Globally disable ANSI color output, overriding auto-detection.
///
/// Call this early in `main()` to suppress all ANSI codes. Useful for
/// CLI apps with a `--no-color` flag.
///
/// # Examples
///
/// ```
/// nanocolor::disable();
/// ```
pub fn disable() {
    COLOR_MODE.store(2, AtomicOrdering::SeqCst);
}

// Test override: 0 = no override, 1 = active
static TEST_OVERRIDE: AtomicU8 = AtomicU8::new(0);
static TEST_OVERRIDE_VALUE: AtomicBool = AtomicBool::new(false);
static COLOR_OVERRIDE_LOCK: Mutex<()> = Mutex::new(());

/// Force colors on or off for testing purposes.
///
/// This sets a separate test-level override that takes effect after
/// `enable()`/`disable()`. Useful for downstream integration tests
/// that need deterministic color output.
///
/// # Examples
///
/// ```
/// nanocolor::set_colors_override(true);
/// // ... assert ANSI output ...
/// nanocolor::clear_colors_override();
/// ```
pub fn set_colors_override(enabled: bool) {
    TEST_OVERRIDE_VALUE.store(enabled, AtomicOrdering::SeqCst);
    TEST_OVERRIDE.store(1, AtomicOrdering::SeqCst);
}

/// Clear the test override so the real detection is used.
///
/// # Examples
///
/// ```
/// nanocolor::set_colors_override(true);
/// // ... test ...
/// nanocolor::clear_colors_override();
/// ```
pub fn clear_colors_override() {
    TEST_OVERRIDE.store(0, AtomicOrdering::SeqCst);
}

/// Run a closure with the color override set, holding a lock to prevent
/// races with other tests that also use color overrides.
///
/// The override is automatically cleared when the closure returns.
///
/// # Examples
///
/// ```
/// use nanocolor::Colorize;
/// let output = nanocolor::with_colors_override(true, || {
///     format!("{}", nanocolor::style("hello").red())
/// });
/// assert_eq!(output, "\x1b[31mhello\x1b[0m");
/// ```
pub fn with_colors_override<R>(enabled: bool, f: impl FnOnce() -> R) -> R {
    let _guard = COLOR_OVERRIDE_LOCK.lock().unwrap();
    set_colors_override(enabled);
    let result = f();
    clear_colors_override();
    result
}

/// The 16 standard ANSI terminal colors.
///
/// Includes 8 standard colors (SGR codes 30–37) and 8 bright variants (SGR codes 90–97).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Color {
    /// Returns the ANSI SGR foreground code for this color (30–37 or 90–97).
    pub fn fg_code(self) -> u8 {
        match self {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::BrightBlack => 90,
            Color::BrightRed => 91,
            Color::BrightGreen => 92,
            Color::BrightYellow => 93,
            Color::BrightBlue => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan => 96,
            Color::BrightWhite => 97,
        }
    }

    /// Returns the ANSI SGR background code for this color (foreground code + 10).
    pub fn bg_code(self) -> u8 {
        self.fg_code() + 10
    }
}

/// Text style modifiers (bold, dim, italic, underline, strikethrough).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    RapidBlink,
    Reverse,
    Hidden,
    Strikethrough,
    Overline,
}

impl Style {
    /// Returns the ANSI SGR code for this style.
    pub fn code(self) -> u8 {
        match self {
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
        }
    }
}

/// A string with accumulated ANSI color and style information.
///
/// Created by calling [`Colorize`] methods on `&str`, `String`, primitive types,
/// or via the [`style()`] helper. Implements [`Display`](std::fmt::Display) to
/// emit ANSI escape codes when styling is active, or plain text otherwise.
///
/// Use [`.whenever()`](Self::whenever) to conditionally disable styling per-value,
/// and [`.mask()`](Self::mask) to hide decorative content when colors are off.
pub struct StyledString<'a> {
    pub text: Cow<'a, str>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub styles: Vec<Style>,
    pub(crate) condition: Option<bool>,
    pub(crate) masked: bool,
}

/// Trait providing chainable color and style methods.
///
/// Implemented for `&str`, `String`, [`StyledString`], and all common primitive
/// types (`i8`–`i128`, `u8`–`u128`, `f32`, `f64`, `bool`, `char`). Methods can
/// be chained in any order to combine foreground color, background color, and
/// text styles.
///
/// For custom `Display` types, use the [`style()`] helper function.
///
/// # Examples
///
/// ```
/// use nanocolor::Colorize;
///
/// let styled = "hello".red().bold();
/// let highlighted = "note".bright_cyan().on_black().underline();
/// let count = 42.green();
/// ```
macro_rules! define_fg_methods {
    ($($method:ident => $color:ident),* $(,)?) => {
        $(fn $method(self) -> StyledString<'static> where Self: Sized {
            let mut s = self.styled(); s.fg = Some(Color::$color); s
        })*
    };
}

macro_rules! define_bg_methods {
    ($($method:ident => $color:ident),* $(,)?) => {
        $(fn $method(self) -> StyledString<'static> where Self: Sized {
            let mut s = self.styled(); s.bg = Some(Color::$color); s
        })*
    };
}

macro_rules! define_style_methods {
    ($($method:ident => $style:ident),* $(,)?) => {
        $(fn $method(self) -> StyledString<'static> where Self: Sized {
            let mut s = self.styled(); s.styles.push(Style::$style); s
        })*
    };
}

pub trait Colorize {
    /// Wraps the value in a [`StyledString`] with no colors or styles applied.
    fn styled(self) -> StyledString<'static>;

    define_fg_methods! {
        black => Black, red => Red, green => Green, yellow => Yellow,
        blue => Blue, magenta => Magenta, cyan => Cyan, white => White,
        bright_black => BrightBlack, bright_red => BrightRed,
        bright_green => BrightGreen, bright_yellow => BrightYellow,
        bright_blue => BrightBlue, bright_magenta => BrightMagenta,
        bright_cyan => BrightCyan, bright_white => BrightWhite,
    }

    define_bg_methods! {
        on_black => Black, on_red => Red, on_green => Green, on_yellow => Yellow,
        on_blue => Blue, on_magenta => Magenta, on_cyan => Cyan, on_white => White,
        on_bright_black => BrightBlack, on_bright_red => BrightRed,
        on_bright_green => BrightGreen, on_bright_yellow => BrightYellow,
        on_bright_blue => BrightBlue, on_bright_magenta => BrightMagenta,
        on_bright_cyan => BrightCyan, on_bright_white => BrightWhite,
    }

    define_style_methods! {
        bold => Bold, dim => Dim, italic => Italic, underline => Underline,
        blink => Blink, rapid_blink => RapidBlink, reverse => Reverse,
        hidden => Hidden, strikethrough => Strikethrough, overline => Overline,
    }
}

impl Colorize for &str {
    fn styled(self) -> StyledString<'static> {
        StyledString {
            text: Cow::Owned(self.to_owned()),
            fg: None,
            bg: None,
            styles: Vec::new(),
            condition: None,
            masked: false,
        }
    }
}

impl Colorize for String {
    fn styled(self) -> StyledString<'static> {
        StyledString {
            text: Cow::Owned(self),
            fg: None,
            bg: None,
            styles: Vec::new(),
            condition: None,
            masked: false,
        }
    }
}

impl<'a> Colorize for StyledString<'a> {
    fn styled(self) -> StyledString<'static> {
        StyledString {
            text: Cow::Owned(self.text.into_owned()),
            fg: self.fg,
            bg: self.bg,
            styles: self.styles,
            condition: self.condition,
            masked: self.masked,
        }
    }
}

impl<'a> StyledString<'a> {
    /// Conditionally enable or disable styling for this value.
    ///
    /// When `condition` is `false`, ANSI codes are suppressed regardless of the
    /// global [`colors_enabled()`] state. When `true`, the global state applies
    /// normally (styling is not forced on).
    ///
    /// If called multiple times, the last call wins. If never called, styling
    /// follows the global state.
    ///
    /// # Examples
    ///
    /// ```
    /// use nanocolor::Colorize;
    ///
    /// let is_tty = true;
    /// println!("{}", "warning".yellow().whenever(is_tty));
    ///
    /// // Always plain text, even if colors are globally enabled
    /// println!("{}", "no color".red().whenever(false));
    /// ```
    pub fn whenever(mut self, condition: bool) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Mark this value as decorative so it renders as an empty string when
    /// styling is inactive.
    ///
    /// When styling is active (colors enabled, no `.whenever(false)` override),
    /// masked values render normally with ANSI codes. When styling is inactive,
    /// they produce an empty string instead of plain text.
    ///
    /// This is useful for emoji, symbols, or other decorative text that should
    /// be hidden in non-TTY or piped output.
    ///
    /// # Examples
    ///
    /// ```
    /// use nanocolor::Colorize;
    ///
    /// // Renders "✓ " with green ANSI when colors are on, "" when off
    /// print!("{}", "✓ ".green().mask());
    /// println!("done");
    /// ```
    pub fn mask(mut self) -> Self {
        self.masked = true;
        self
    }
}

macro_rules! impl_colorize {
    ($($t:ty),*) => {
        $(
            impl Colorize for $t {
                fn styled(self) -> StyledString<'static> {
                    StyledString {
                        text: Cow::Owned(self.to_string()),
                        fg: None,
                        bg: None,
                        styles: Vec::new(),
                        condition: None,
                        masked: false,
                    }
                }
            }
        )*
    };
}

impl_colorize!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool, char
);

/// Wraps any `Display` type in a `StyledString` for chainable styling.
///
/// Use this for custom types that aren't covered by the built-in `Colorize` impls:
/// ```
/// # use nanocolor::{style, Colorize};
/// struct MyId(u64);
/// impl std::fmt::Display for MyId {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "ID-{}", self.0)
///     }
/// }
/// let styled = style(MyId(42)).red().bold();
/// ```
pub fn style<T: std::fmt::Display>(value: T) -> StyledString<'static> {
    StyledString {
        text: Cow::Owned(value.to_string()),
        fg: None,
        bg: None,
        styles: Vec::new(),
        condition: None,
        masked: false,
    }
}

// Platform-specific TTY detection via raw FFI (no dependencies)
#[cfg(unix)]
fn isatty_stdout() -> bool {
    extern "C" {
        fn isatty(fd: i32) -> i32;
    }
    // SAFETY: isatty is a standard POSIX function, fd 1 is stdout
    unsafe { isatty(1) != 0 }
}

#[cfg(windows)]
fn isatty_stdout() -> bool {
    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> *mut core::ffi::c_void;
        fn GetConsoleMode(hConsoleHandle: *mut core::ffi::c_void, lpMode: *mut u32) -> i32;
    }
    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5; // (DWORD)-11
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle.is_null() {
            return false;
        }
        let mut mode: u32 = 0;
        GetConsoleMode(handle, &mut mode) != 0
    }
}

#[cfg(not(any(unix, windows)))]
fn isatty_stdout() -> bool {
    false
}

/// Returns whether ANSI color output is currently enabled.
///
/// Checks the `NO_COLOR` environment variable first (any non-empty value disables
/// colors), then falls back to TTY detection on stdout. The result is cached for
/// the lifetime of the process via `OnceLock`.
pub fn colors_enabled() -> bool {
    // Public enable()/disable() override takes priority
    match COLOR_MODE.load(AtomicOrdering::SeqCst) {
        1 => return true,
        2 => return false,
        _ => {}
    }

    {
        let ov = TEST_OVERRIDE.load(AtomicOrdering::SeqCst);
        if ov != 0 {
            return TEST_OVERRIDE_VALUE.load(AtomicOrdering::SeqCst);
        }
    }

    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        // NO_COLOR takes precedence (no-color.org standard)
        if std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()) {
            return false;
        }
        isatty_stdout()
    })
}

impl<'a> std::fmt::Display for StyledString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let styling_active = match self.condition {
            Some(false) => false,
            _ => colors_enabled(),
        };

        if self.masked && !styling_active {
            return Ok(());
        }

        if !styling_active || (self.fg.is_none() && self.bg.is_none() && self.styles.is_empty()) {
            return f.write_str(&self.text);
        }

        write!(f, "\x1b[")?;
        let mut first = true;
        for style in &self.styles {
            if !first {
                write!(f, ";")?;
            }
            write!(f, "{}", style.code())?;
            first = false;
        }
        if let Some(fg) = self.fg {
            if !first {
                write!(f, ";")?;
            }
            write!(f, "{}", fg.fg_code())?;
            first = false;
        }
        if let Some(bg) = self.bg {
            if !first {
                write!(f, ";")?;
            }
            write!(f, "{}", bg.bg_code())?;
        }
        write!(f, "m{}\x1b[0m", self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreground_colors() {
        let cases: Vec<(fn(&str) -> StyledString<'static>, Color)> = vec![
            (|s| s.black(), Color::Black),
            (|s| s.red(), Color::Red),
            (|s| s.green(), Color::Green),
            (|s| s.yellow(), Color::Yellow),
            (|s| s.blue(), Color::Blue),
            (|s| s.magenta(), Color::Magenta),
            (|s| s.cyan(), Color::Cyan),
            (|s| s.white(), Color::White),
            (|s| s.bright_black(), Color::BrightBlack),
            (|s| s.bright_red(), Color::BrightRed),
            (|s| s.bright_green(), Color::BrightGreen),
            (|s| s.bright_yellow(), Color::BrightYellow),
            (|s| s.bright_blue(), Color::BrightBlue),
            (|s| s.bright_magenta(), Color::BrightMagenta),
            (|s| s.bright_cyan(), Color::BrightCyan),
            (|s| s.bright_white(), Color::BrightWhite),
        ];
        for (method, expected_color) in cases {
            let styled = method("hi");
            assert_eq!(styled.fg, Some(expected_color));
            assert_eq!(styled.bg, None);
            assert!(styled.styles.is_empty());
        }
    }

    #[test]
    fn test_background_colors() {
        let cases: Vec<(fn(&str) -> StyledString<'static>, Color)> = vec![
            (|s| s.on_black(), Color::Black),
            (|s| s.on_red(), Color::Red),
            (|s| s.on_green(), Color::Green),
            (|s| s.on_yellow(), Color::Yellow),
            (|s| s.on_blue(), Color::Blue),
            (|s| s.on_magenta(), Color::Magenta),
            (|s| s.on_cyan(), Color::Cyan),
            (|s| s.on_white(), Color::White),
            (|s| s.on_bright_black(), Color::BrightBlack),
            (|s| s.on_bright_red(), Color::BrightRed),
            (|s| s.on_bright_green(), Color::BrightGreen),
            (|s| s.on_bright_yellow(), Color::BrightYellow),
            (|s| s.on_bright_blue(), Color::BrightBlue),
            (|s| s.on_bright_magenta(), Color::BrightMagenta),
            (|s| s.on_bright_cyan(), Color::BrightCyan),
            (|s| s.on_bright_white(), Color::BrightWhite),
        ];
        for (method, expected_color) in cases {
            let styled = method("hi");
            assert_eq!(styled.fg, None);
            assert_eq!(styled.bg, Some(expected_color));
            assert!(styled.styles.is_empty());
        }
    }

    #[test]
    fn test_style_methods() {
        let cases: Vec<(fn(&str) -> StyledString<'static>, Style)> = vec![
            (|s| s.bold(), Style::Bold),
            (|s| s.dim(), Style::Dim),
            (|s| s.italic(), Style::Italic),
            (|s| s.underline(), Style::Underline),
            (|s| s.blink(), Style::Blink),
            (|s| s.rapid_blink(), Style::RapidBlink),
            (|s| s.reverse(), Style::Reverse),
            (|s| s.hidden(), Style::Hidden),
            (|s| s.strikethrough(), Style::Strikethrough),
            (|s| s.overline(), Style::Overline),
        ];
        for (method, expected_style) in cases {
            let styled = method("hi");
            assert_eq!(styled.fg, None);
            assert_eq!(styled.bg, None);
            assert_eq!(styled.styles, vec![expected_style]);
        }
    }

    #[test]
    fn test_chaining_order_independence() {
        let a = "hi".bold().italic();
        let b = "hi".italic().bold();
        // Both should contain Bold and Italic regardless of order
        assert!(a.styles.contains(&Style::Bold));
        assert!(a.styles.contains(&Style::Italic));
        assert!(b.styles.contains(&Style::Bold));
        assert!(b.styles.contains(&Style::Italic));
    }

    #[test]
    fn test_string_impl() {
        let s = String::from("hello").red();
        assert_eq!(s.fg, Some(Color::Red));
        assert_eq!(&*s.text, "hello");
    }

    #[test]
    fn test_display_colors_forced_on() {
        with_colors_override(true, || {
            let s = "hello".red();
            let output = format!("{}", s);
            assert_eq!(output, "\x1b[31mhello\x1b[0m");
        });
    }

    #[test]
    fn test_display_colors_forced_off() {
        with_colors_override(false, || {
            let s = "hello".red().bold();
            let output = format!("{}", s);
            assert_eq!(output, "hello");
        });
    }

    #[test]
    fn test_display_plain_styled_string() {
        with_colors_override(true, || {
            let s = "plain".styled();
            let output = format!("{}", s);
            // No colors/styles applied, should be plain text even when colors enabled
            assert_eq!(output, "plain");
        });
    }

    #[test]
    fn test_display_empty_string() {
        with_colors_override(true, || {
            let s = "".red();
            let output = format!("{}", s);
            assert_eq!(output, "\x1b[31m\x1b[0m");
        });
    }

    #[test]
    fn test_display_special_characters() {
        with_colors_override(true, || {
            let s = "hello\nworld\ttab".green();
            let output = format!("{}", s);
            assert_eq!(output, "\x1b[32mhello\nworld\ttab\x1b[0m");
        });
    }

    #[test]
    fn test_display_combined_fg_bg_style() {
        with_colors_override(true, || {
            let s = "hi".bold().red().on_blue();
            let output = format!("{}", s);
            assert_eq!(output, "\x1b[1;31;44mhi\x1b[0m");
        });
    }

    #[test]
    fn test_integer_red() {
        let s = 42.red();
        assert_eq!(&*s.text, "42");
        assert_eq!(s.fg, Some(Color::Red));
        assert_eq!(s.bg, None);
        assert!(s.styles.is_empty());
    }

    #[test]
    fn test_float_green() {
        let s = 3.14_f64.green();
        assert_eq!(&*s.text, "3.14");
        assert_eq!(s.fg, Some(Color::Green));
    }

    #[test]
    fn test_bool_bold() {
        let s = true.bold();
        assert_eq!(&*s.text, "true");
        assert_eq!(s.styles, vec![Style::Bold]);
        assert_eq!(s.fg, None);
    }

    #[test]
    fn test_char_cyan() {
        let s = 'x'.cyan();
        assert_eq!(&*s.text, "x");
        assert_eq!(s.fg, Some(Color::Cyan));
    }

    #[test]
    fn test_style_helper_function() {
        // Test with a formatted string value (simulating a custom Display type)
        let s = style(format!("ID-{}", 42)).red().bold();
        assert_eq!(&*s.text, "ID-42");
        assert_eq!(s.fg, Some(Color::Red));
        assert_eq!(s.styles, vec![Style::Bold]);
    }

    #[test]
    fn test_primitive_chaining_matches_str() {
        // Requirement 1.3: primitives support same chaining as &str
        let s = 99.red().on_blue().bold();
        assert_eq!(&*s.text, "99");
        assert_eq!(s.fg, Some(Color::Red));
        assert_eq!(s.bg, Some(Color::Blue));
        assert_eq!(s.styles, vec![Style::Bold]);
    }

    #[test]
    fn test_str_still_works() {
        // Requirement 1.4: backwards compatibility
        let s = "hello".red();
        assert_eq!(&*s.text, "hello");
        assert_eq!(s.fg, Some(Color::Red));

        let s2 = String::from("world").green();
        assert_eq!(&*s2.text, "world");
        assert_eq!(s2.fg, Some(Color::Green));
    }

    #[test]
    fn test_whenever_false_with_colors_forced_on() {
        // Req 2.3: .whenever(false) suppresses ANSI regardless of global state
        with_colors_override(true, || {
            let output = format!("{}", "hello".red().whenever(false));
            assert_eq!(output, "hello");
        });
    }

    #[test]
    fn test_whenever_true_with_colors_forced_off() {
        // Req 2.2: .whenever(true) defers to colors_enabled(), which is false here
        with_colors_override(false, || {
            let output = format!("{}", "hello".red().whenever(true));
            assert_eq!(output, "hello");
        });
    }

    #[test]
    fn test_whenever_false_before_color() {
        // Req 2.6: .whenever() is chainable in any order
        with_colors_override(true, || {
            let output = format!("{}", "hi".styled().whenever(false).red());
            assert_eq!(output, "hi");
        });
    }

    #[test]
    fn test_whenever_true_after_color_with_colors_on() {
        // Req 2.2, 2.6: .whenever(true) with colors on renders ANSI
        with_colors_override(true, || {
            let output = format!("{}", "hi".red().whenever(true));
            assert_eq!(output, "\x1b[31mhi\x1b[0m");
        });
    }

    #[test]
    fn test_whenever_last_write_wins() {
        // Req 2.5: last .whenever() call determines the condition
        with_colors_override(true, || {
            let output = format!("{}", "hi".red().whenever(true).whenever(false));
            assert_eq!(output, "hi");
        });
    }

    #[test]
    fn test_mask_colors_off_renders_empty() {
        // Req 3.3: masked value with colors off renders empty string
        with_colors_override(false, || {
            let output = format!("{}", "✓".green().mask());
            assert_eq!(output, "");
        });
    }

    #[test]
    fn test_mask_colors_on_renders_ansi() {
        // Req 3.2: masked value with colors on renders ANSI-wrapped
        with_colors_override(true, || {
            let output = format!("{}", "✓".green().mask());
            assert_eq!(output, "\x1b[32m✓\x1b[0m");
        });
    }

    #[test]
    fn test_mask_whenever_false_renders_empty() {
        // Req 3.4: .mask().whenever(false) always renders empty
        with_colors_override(true, || {
            let output = format!("{}", "decorative".red().mask().whenever(false));
            assert_eq!(output, "");
        });
    }

    #[test]
    fn test_mask_whenever_true_colors_on_renders_normally() {
        // Req 3.5, 3.6: .mask().whenever(true) with colors on renders ANSI
        with_colors_override(true, || {
            let output = format!("{}", "decorative".red().mask().whenever(true));
            assert_eq!(output, "\x1b[31mdecorative\x1b[0m");
        });
    }

    #[test]
    fn test_enable_forces_colors_on() {
        let _guard = COLOR_OVERRIDE_LOCK.lock().unwrap();
        clear_colors_override();
        enable();
        let output = format!("{}", "hi".red());
        assert_eq!(output, "\x1b[31mhi\x1b[0m");
        COLOR_MODE.store(0, AtomicOrdering::SeqCst);
    }

    #[test]
    fn test_disable_forces_colors_off() {
        let _guard = COLOR_OVERRIDE_LOCK.lock().unwrap();
        clear_colors_override();
        disable();
        let output = format!("{}", "hi".red());
        assert_eq!(output, "hi");
        COLOR_MODE.store(0, AtomicOrdering::SeqCst);
    }
}
