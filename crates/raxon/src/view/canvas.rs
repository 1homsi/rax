//! A low-level vector [`canvas`] plus high-level [`charts`] builders.
//!
//! The chart builders compute their geometry as a pure function of the data
//! and a fixed size, returning a `Vec<DrawCmd>`. That layout math is
//! host-testable; the iOS backend just renders the commands. Charts are sized
//! explicitly (`width`, `height`) — drop one into a fixed-height card.
//!
//! ```no_run
//! # use raxon::view::charts::bar_chart;
//! bar_chart(&[4.0, 8.0, 6.0, 10.0], raxon::core::Color::hex(0x4583C4ff), 280.0, 160.0);
//! ```

use crate::core::{Dimension, LayoutStyle};
use crate::dom::{Attribute, DrawCmd, Tree, WidgetId};

use super::view::View;

/// A vector drawing surface of a fixed size. Build via [`canvas`] or one of the
/// [`charts`] builders.
pub struct Canvas {
    width: f32,
    height: f32,
    cmds: Vec<DrawCmd>,
}

/// Creates a `width`×`height` canvas that renders `cmds` (in the canvas's local
/// coordinate space, origin top-left).
pub fn canvas(width: f32, height: f32, cmds: Vec<DrawCmd>) -> Canvas {
    Canvas { width, height, cmds }
}

impl View for Canvas {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_canvas();
        let style = LayoutStyle {
            width: Dimension::Points(self.width),
            height: Dimension::Points(self.height),
            ..LayoutStyle::default()
        };
        tree.set_style(id, style);
        tree.set(id, Attribute::DrawList(self.cmds));
        id
    }
}

/// Chart builders: pure data→[`DrawCmd`] layout functions plus thin [`View`]
/// wrappers around [`canvas`].
pub mod charts {
    use super::{canvas, Canvas};
    use crate::core::Color;
    use crate::dom::{DrawCmd, Stroke, TextAlign};

    /// Inner padding (points) reserved around the plotting area for every chart.
    const PAD: f32 = 8.0;

    fn plot_rect(w: f32, h: f32) -> (f32, f32, f32, f32) {
        // (x, y, width, height) of the drawable plot area.
        let pw = (w - 2.0 * PAD).max(0.0);
        let ph = (h - 2.0 * PAD).max(0.0);
        (PAD, PAD, pw, ph)
    }

    fn max_or_one(values: &[f32]) -> f32 {
        values
            .iter()
            .cloned()
            .fold(0.0_f32, f32::max)
            .max(f32::MIN_POSITIVE)
    }

    /// Compute the [`DrawCmd`]s for a vertical bar chart of `values`.
    ///
    /// Bars are evenly spaced across the plot width, scaled so the largest value
    /// fills the plot height. Returns one rounded [`DrawCmd::Rect`] per value.
    pub fn bar_chart_cmds(values: &[f32], color: Color, w: f32, h: f32) -> Vec<DrawCmd> {
        if values.is_empty() {
            return Vec::new();
        }
        let (px, _py, pw, ph) = plot_rect(w, h);
        let baseline = h - PAD;
        let max = max_or_one(values);
        let slot = pw / values.len() as f32;
        let bar_w = (slot * 0.7).max(1.0);
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let bar_h = (v.max(0.0) / max) * ph;
                let x = px + i as f32 * slot + (slot - bar_w) / 2.0;
                DrawCmd::Rect {
                    x,
                    y: baseline - bar_h,
                    w: bar_w,
                    h: bar_h,
                    radius: (bar_w * 0.2).min(4.0),
                    fill: Some(color),
                    stroke: None,
                }
            })
            .collect()
    }

    fn points_for(values: &[f32], w: f32, h: f32) -> Vec<(f32, f32)> {
        let (px, _py, pw, ph) = plot_rect(w, h);
        let baseline = h - PAD;
        let max = max_or_one(values);
        let n = values.len();
        let step = if n > 1 { pw / (n as f32 - 1.0) } else { 0.0 };
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = if n > 1 { px + i as f32 * step } else { px + pw / 2.0 };
                let y = baseline - (v.max(0.0) / max) * ph;
                (x, y)
            })
            .collect()
    }

    /// Compute the [`DrawCmd`]s for a line chart of `values`: a stroked polyline
    /// plus a small dot at each data point.
    pub fn line_chart_cmds(values: &[f32], color: Color, w: f32, h: f32) -> Vec<DrawCmd> {
        if values.is_empty() {
            return Vec::new();
        }
        let pts = points_for(values, w, h);
        let mut cmds = vec![DrawCmd::Path {
            points: pts.clone(),
            closed: false,
            fill: None,
            stroke: Some(Stroke::new(2.0, color)),
        }];
        for (x, y) in pts {
            cmds.push(DrawCmd::Circle {
                cx: x,
                cy: y,
                r: 3.0,
                fill: Some(color),
                stroke: None,
            });
        }
        cmds
    }

    /// Compute the [`DrawCmd`]s for an area chart of `values`: a translucent
    /// filled region under the line, plus the stroked line on top.
    pub fn area_chart_cmds(values: &[f32], color: Color, w: f32, h: f32) -> Vec<DrawCmd> {
        if values.is_empty() {
            return Vec::new();
        }
        let pts = points_for(values, w, h);
        let baseline = h - PAD;
        let mut fill_pts = pts.clone();
        if let (Some(&(first_x, _)), Some(&(last_x, _))) = (pts.first(), pts.last()) {
            fill_pts.push((last_x, baseline));
            fill_pts.push((first_x, baseline));
        }
        vec![
            DrawCmd::Path {
                points: fill_pts,
                closed: true,
                fill: Some(color.with_alpha(60)),
                stroke: None,
            },
            DrawCmd::Path {
                points: pts,
                closed: false,
                fill: None,
                stroke: Some(Stroke::new(2.0, color)),
            },
        ]
    }

    /// Compute the [`DrawCmd`]s for a pie chart from `(value, color)` slices.
    ///
    /// Each slice is approximated as a filled polygon (a fan of points along its
    /// arc). Starts at 12 o'clock and sweeps clockwise.
    pub fn pie_chart_cmds(slices: &[(f32, Color)], w: f32, h: f32) -> Vec<DrawCmd> {
        let total: f32 = slices.iter().map(|(v, _)| v.max(0.0)).sum();
        if total <= 0.0 {
            return Vec::new();
        }
        let cx = w / 2.0;
        let cy = h / 2.0;
        let radius = (w.min(h) / 2.0 - PAD).max(0.0);
        let tau = std::f32::consts::TAU;
        let mut angle = -std::f32::consts::FRAC_PI_2; // start at top (12 o'clock)
        let mut cmds = Vec::with_capacity(slices.len());
        for (value, color) in slices {
            let frac = value.max(0.0) / total;
            let sweep = frac * tau;
            // Enough segments for a smooth arc, scaled by the slice's size.
            let segments = ((frac * 64.0).ceil() as usize).max(2);
            let mut points = Vec::with_capacity(segments + 2);
            points.push((cx, cy));
            for s in 0..=segments {
                let a = angle + sweep * (s as f32 / segments as f32);
                points.push((cx + radius * a.cos(), cy + radius * a.sin()));
            }
            cmds.push(DrawCmd::Path {
                points,
                closed: true,
                fill: Some(*color),
                stroke: None,
            });
            angle += sweep;
        }
        cmds
    }

    /// A vertical bar chart of `values` at the given size.
    pub fn bar_chart(values: &[f32], color: Color, w: f32, h: f32) -> Canvas {
        canvas(w, h, bar_chart_cmds(values, color, w, h))
    }

    /// A line chart of `values` at the given size.
    pub fn line_chart(values: &[f32], color: Color, w: f32, h: f32) -> Canvas {
        canvas(w, h, line_chart_cmds(values, color, w, h))
    }

    /// An area chart of `values` at the given size.
    pub fn area_chart(values: &[f32], color: Color, w: f32, h: f32) -> Canvas {
        canvas(w, h, area_chart_cmds(values, color, w, h))
    }

    /// A pie chart from `(value, color)` slices at the given size.
    pub fn pie_chart(slices: &[(f32, Color)], w: f32, h: f32) -> Canvas {
        canvas(w, h, pie_chart_cmds(slices, w, h))
    }

    /// The chart's color cycle for multi-series/category charts: a small, legible
    /// palette callers can index into.
    pub const PALETTE: [Color; 6] = [
        Color::rgb(0x45, 0x83, 0xC4),
        Color::rgb(0x2E, 0xC4, 0x9A),
        Color::rgb(0xF5, 0xA6, 0x23),
        Color::rgb(0x9B, 0x59, 0xD6),
        Color::rgb(0xE0, 0x4F, 0x7A),
        Color::rgb(0x3A, 0xB0, 0xC9),
    ];

    /// A single text label drawn at `(x, y)` — handy for annotating a canvas.
    pub fn label(x: f32, y: f32, text: impl Into<String>, size: f32, color: Color) -> DrawCmd {
        DrawCmd::Text {
            x,
            y,
            text: text.into(),
            size,
            color,
            align: TextAlign::Start,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const W: f32 = 200.0;
        const H: f32 = 100.0;
        const C: Color = Color::rgb(0x45, 0x83, 0xC4);

        fn within_bounds(cmds: &[DrawCmd]) -> bool {
            cmds.iter().all(|cmd| match cmd {
                DrawCmd::Rect { x, y, w, h, .. } => {
                    *x >= 0.0 && *y >= 0.0 && x + w <= W + 0.01 && y + h <= H + 0.01
                }
                DrawCmd::Circle { cx, cy, .. } => *cx >= 0.0 && *cy >= 0.0 && *cx <= W && *cy <= H,
                DrawCmd::Path { points, .. } => points
                    .iter()
                    .all(|(x, y)| *x >= -0.01 && *y >= -0.01 && *x <= W + 0.01 && *y <= H + 0.01),
                _ => true,
            })
        }

        #[test]
        fn bar_chart_emits_one_rect_per_value_within_bounds() {
            let cmds = bar_chart_cmds(&[4.0, 8.0, 6.0, 10.0], C, W, H);
            assert_eq!(cmds.len(), 4);
            assert!(within_bounds(&cmds));
            // The tallest bar belongs to the largest value (10.0 at index 3).
            let heights: Vec<f32> = cmds
                .iter()
                .map(|c| match c {
                    DrawCmd::Rect { h, .. } => *h,
                    _ => 0.0,
                })
                .collect();
            let tallest = heights
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0;
            assert_eq!(tallest, 3);
        }

        #[test]
        fn empty_data_produces_no_commands() {
            assert!(bar_chart_cmds(&[], C, W, H).is_empty());
            assert!(line_chart_cmds(&[], C, W, H).is_empty());
            assert!(area_chart_cmds(&[], C, W, H).is_empty());
            assert!(pie_chart_cmds(&[], W, H).is_empty());
        }

        #[test]
        fn all_zero_values_do_not_divide_by_zero() {
            let cmds = bar_chart_cmds(&[0.0, 0.0, 0.0], C, W, H);
            assert_eq!(cmds.len(), 3);
            assert!(cmds.iter().all(|c| matches!(c, DrawCmd::Rect { h, .. } if *h == 0.0)));
        }

        #[test]
        fn line_chart_has_polyline_plus_a_dot_per_point() {
            let cmds = line_chart_cmds(&[1.0, 2.0, 3.0], C, W, H);
            assert!(within_bounds(&cmds));
            let paths = cmds.iter().filter(|c| matches!(c, DrawCmd::Path { .. })).count();
            let dots = cmds.iter().filter(|c| matches!(c, DrawCmd::Circle { .. })).count();
            assert_eq!(paths, 1);
            assert_eq!(dots, 3);
            // The polyline must thread through exactly the data points.
            if let DrawCmd::Path { points, closed, .. } = &cmds[0] {
                assert_eq!(points.len(), 3);
                assert!(!closed);
            } else {
                panic!("first command should be the polyline");
            }
        }

        #[test]
        fn area_chart_closes_its_fill_to_the_baseline() {
            let cmds = area_chart_cmds(&[2.0, 5.0, 1.0], C, W, H);
            assert_eq!(cmds.len(), 2);
            match &cmds[0] {
                DrawCmd::Path { points, closed, fill, stroke } => {
                    assert!(closed);
                    assert!(fill.is_some());
                    assert!(stroke.is_none());
                    // 3 data points + 2 baseline corners.
                    assert_eq!(points.len(), 5);
                }
                _ => panic!("first command should be the filled area"),
            }
        }

        #[test]
        fn pie_slices_sum_to_full_circle_and_stay_in_bounds() {
            let slices = [
                (3.0, Color::rgb(255, 0, 0)),
                (1.0, Color::rgb(0, 255, 0)),
                (1.0, Color::rgb(0, 0, 255)),
            ];
            let cmds = pie_chart_cmds(&slices, H, H);
            assert_eq!(cmds.len(), 3);
            assert!(cmds.iter().all(|c| matches!(c, DrawCmd::Path { closed: true, fill: Some(_), .. })));
            // Every polygon vertex must sit inside the square canvas.
            let r = H / 2.0;
            for cmd in &cmds {
                if let DrawCmd::Path { points, .. } = cmd {
                    for (x, y) in points {
                        let d = ((x - r).powi(2) + (y - r).powi(2)).sqrt();
                        assert!(d <= r + 0.01, "vertex outside the pie radius");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod widget_tests {
    use super::charts::bar_chart;
    use crate::core::Color;
    use crate::dom::{Attribute, Host, Mutation, RecordingBackend, Tree};
    use crate::reactive::create_root;
    use crate::view::{mount, text, MenuItem, ViewExt};

    #[test]
    fn bar_chart_widget_emits_a_drawlist_attribute() {
        let backend = RecordingBackend::new();
        let log = backend.log();
        let (_root, scope) = create_root(|| {
            let mut tree = Tree::new(Host::new(backend));
            mount(
                &mut tree,
                bar_chart(&[1.0, 2.0, 3.0], Color::rgb(0, 0, 0), 120.0, 80.0),
            )
        });
        let has_drawlist = log.borrow().iter().any(|m| {
            matches!(
                m,
                Mutation::SetAttribute { attr: Attribute::DrawList(cmds), .. } if cmds.len() == 3
            )
        });
        assert!(has_drawlist, "canvas should emit a 3-rect DrawList");
        scope.dispose();
    }

    #[test]
    fn context_menu_modifier_emits_the_attribute() {
        let backend = RecordingBackend::new();
        let log = backend.log();
        let (_root, scope) = create_root(|| {
            let mut tree = Tree::new(Host::new(backend));
            mount(
                &mut tree,
                text("Row").context_menu(vec![
                    MenuItem::new("Rename", || {}).icon("pencil"),
                    MenuItem::new("Delete", || {}).destructive(),
                ]),
            )
        });
        let menu = log.borrow().iter().find_map(|m| match m {
            Mutation::SetAttribute { attr: Attribute::ContextMenu(items), .. } => Some(items.clone()),
            _ => None,
        });
        let menu = menu.expect("context menu attribute recorded");
        assert_eq!(menu.len(), 2);
        assert_eq!(menu[0].title, "Rename");
        assert_eq!(menu[0].icon.as_deref(), Some("pencil"));
        assert!(!menu[0].destructive);
        assert!(menu[1].destructive);
        scope.dispose();
    }
}
