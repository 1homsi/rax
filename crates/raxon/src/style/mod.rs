//! Typed design tokens and a runtime-switchable [`Theme`] for `rax`.
//!
//! A theme is plain, typed Rust data (autocomplete + compile-time checks for your
//! design system). It is provided down the tree as a `Signal<Theme>`, so:
//!
//! - widgets read tokens via [`theme`] (a tracked read) and update automatically
//!   when the theme changes, and
//! - switching light/dark or brand themes is a single [`Signal::set`] — only the
//!   views that read the changed tokens re-render (fine-grained, no tree diff).
//!
//! ```
//! use raxon::style::{provide_theme, theme, Theme};
//! use raxon::reactive::create_root;
//!
//! let (_, scope) = create_root(|| {
//!     let t = provide_theme(Theme::light());
//!     let _primary = theme().colors.primary; // tracked read
//!     t.set(Theme::dark());                   // runtime switch
//! });
//! scope.dispose();
//! ```

#![forbid(unsafe_code)]

use crate::core::Color;
use crate::reactive::{create_signal, provide_context, use_context, Signal};

/// Semantic color roles. Reference these, not raw colors, so a theme swap
/// recolors the whole app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Primary brand/action color.
    pub primary: Color,
    /// Content on top of `primary`.
    pub on_primary: Color,
    /// Secondary accent.
    pub secondary: Color,
    /// App background.
    pub background: Color,
    /// Raised surface (cards, sheets).
    pub surface: Color,
    /// Primary content on surfaces/background.
    pub on_surface: Color,
    /// Muted/secondary content.
    pub on_surface_muted: Color,
    /// Hairline borders/dividers.
    pub border: Color,
    /// Error/destructive.
    pub error: Color,
    /// Success/positive.
    pub success: Color,
    /// Warning/caution.
    pub warning: Color,
}

/// Spacing scale, in logical points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacing {
    /// Extra small.
    pub xs: f32,
    /// Small.
    pub sm: f32,
    /// Medium (base).
    pub md: f32,
    /// Large.
    pub lg: f32,
    /// Extra large.
    pub xl: f32,
}

/// Corner-radius scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radii {
    /// Small radius.
    pub sm: f32,
    /// Medium radius.
    pub md: f32,
    /// Large radius.
    pub lg: f32,
    /// Fully rounded (pill/circle).
    pub pill: f32,
}

/// Typography size scale, in logical points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Typography {
    /// Caption / fine print.
    pub caption: f32,
    /// Body text.
    pub body: f32,
    /// Title.
    pub title: f32,
    /// Headline.
    pub headline: f32,
    /// Display / hero.
    pub display: f32,
}

/// A complete design theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// Color roles.
    pub colors: Palette,
    /// Spacing scale.
    pub spacing: Spacing,
    /// Radius scale.
    pub radius: Radii,
    /// Typography scale.
    pub typography: Typography,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::light()
    }
}

const SPACING: Spacing = Spacing {
    xs: 4.0,
    sm: 8.0,
    md: 16.0,
    lg: 24.0,
    xl: 40.0,
};
const RADII: Radii = Radii {
    sm: 6.0,
    md: 12.0,
    lg: 20.0,
    pill: 999.0,
};
const TYPOGRAPHY: Typography = Typography {
    caption: 12.0,
    body: 16.0,
    title: 20.0,
    headline: 28.0,
    display: 48.0,
};

impl Theme {
    /// The default light theme.
    pub fn light() -> Theme {
        Theme {
            colors: Palette {
                primary: Color::rgb(40, 90, 220),
                on_primary: Color::WHITE,
                secondary: Color::rgb(90, 100, 120),
                background: Color::rgb(247, 248, 251),
                surface: Color::WHITE,
                on_surface: Color::rgb(22, 24, 35),
                on_surface_muted: Color::rgb(120, 128, 145),
                border: Color::rgb(224, 227, 235),
                error: Color::rgb(214, 60, 60),
                success: Color::rgb(40, 170, 110),
                warning: Color::rgb(220, 160, 40),
            },
            spacing: SPACING,
            radius: RADII,
            typography: TYPOGRAPHY,
        }
    }

    /// The default dark theme.
    pub fn dark() -> Theme {
        Theme {
            colors: Palette {
                primary: Color::rgb(96, 140, 255),
                on_primary: Color::rgb(10, 12, 20),
                secondary: Color::rgb(150, 160, 180),
                background: Color::rgb(16, 18, 24),
                surface: Color::rgb(28, 31, 40),
                on_surface: Color::rgb(232, 235, 242),
                on_surface_muted: Color::rgb(140, 148, 165),
                border: Color::rgb(48, 52, 64),
                error: Color::rgb(240, 100, 100),
                success: Color::rgb(70, 200, 140),
                warning: Color::rgb(240, 185, 70),
            },
            spacing: SPACING,
            radius: RADII,
            typography: TYPOGRAPHY,
        }
    }
}

/// Provides a theme to the current scope and all descendants, returning the
/// `Signal<Theme>` so the app can switch themes at runtime (`signal.set(...)`).
pub fn provide_theme(theme: Theme) -> Signal<Theme> {
    let signal = create_signal(theme);
    provide_context(signal);
    signal
}

/// The theme signal in scope, creating and providing a default light theme if
/// none was provided.
pub fn use_theme() -> Signal<Theme> {
    use_context::<Signal<Theme>>().unwrap_or_else(|| provide_theme(Theme::light()))
}

/// Reads the current theme (a tracked read — callers update when it changes).
pub fn theme() -> Theme {
    use_theme().get()
}

#[cfg(test)]
mod tests;
