//! Web backend foundation.
//!
//! This module provides the pure-Rust half of the WebAssembly DOM backend: a
//! command queue that translates raxon [`Mutation`](crate::dom::Mutation)s into
//! DOM operations. A small wasm host can drain these commands, apply them to the
//! browser DOM, and dispatch browser events back through
//! [`EventSink`](crate::dom::EventSink).

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::{Color, Rect, Size};
use crate::dom::{
    Attribute, Backend, Event, EventSink, GestureKind, HapticStyle, Host, LocalNotification,
    Mutation, WidgetId, WidgetKind,
};
use crate::runtime::App;
use crate::view::View;

/// A shared queue of DOM commands produced by [`WebDomBackend`].
pub type DomCommandQueue = Rc<RefCell<Vec<DomCommand>>>;

/// DOM element kinds used by the first web backend pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomElementKind {
    /// A generic `div` container.
    Div,
    /// Inline text content.
    Span,
    /// A native `button`.
    Button,
    /// An `img` element.
    Image,
    /// An `input type=checkbox`.
    Checkbox,
    /// An `input type=range`.
    Range,
    /// An `input type=text`.
    TextInput,
    /// A scrollable `div`.
    ScrollDiv,
    /// An indeterminate progress indicator.
    ActivityIndicator,
    /// A `progress` element.
    Progress,
    /// A segmented-control host.
    Segmented,
    /// A numeric input used as a stepper.
    Stepper,
    /// Date or datetime input.
    DateInput,
    /// A `textarea`.
    TextArea,
    /// A camera preview video host.
    Video,
    /// An `iframe`.
    Iframe,
    /// A virtualized list host.
    VirtualList,
    /// A map host.
    MapHost,
    /// A `canvas`.
    Canvas,
}

impl DomElementKind {
    /// Maps a framework widget kind to the DOM element used by the web backend.
    pub const fn from_widget_kind(kind: WidgetKind) -> Self {
        match kind {
            WidgetKind::View | WidgetKind::Stack => DomElementKind::Div,
            WidgetKind::Text => DomElementKind::Span,
            WidgetKind::Button => DomElementKind::Button,
            WidgetKind::Image => DomElementKind::Image,
            WidgetKind::Switch => DomElementKind::Checkbox,
            WidgetKind::Slider => DomElementKind::Range,
            WidgetKind::TextInput => DomElementKind::TextInput,
            WidgetKind::Scroll => DomElementKind::ScrollDiv,
            WidgetKind::ActivityIndicator => DomElementKind::ActivityIndicator,
            WidgetKind::Progress => DomElementKind::Progress,
            WidgetKind::Segmented => DomElementKind::Segmented,
            WidgetKind::Stepper => DomElementKind::Stepper,
            WidgetKind::DatePicker => DomElementKind::DateInput,
            WidgetKind::TextArea => DomElementKind::TextArea,
            WidgetKind::Camera => DomElementKind::Video,
            WidgetKind::WebView => DomElementKind::Iframe,
            WidgetKind::LazyList => DomElementKind::VirtualList,
            WidgetKind::MapView => DomElementKind::MapHost,
            WidgetKind::Canvas => DomElementKind::Canvas,
        }
    }

    /// HTML tag name to create for this element kind.
    pub const fn tag_name(self) -> &'static str {
        match self {
            DomElementKind::Div
            | DomElementKind::ScrollDiv
            | DomElementKind::ActivityIndicator
            | DomElementKind::Segmented
            | DomElementKind::VirtualList
            | DomElementKind::MapHost => "div",
            DomElementKind::Span => "span",
            DomElementKind::Button => "button",
            DomElementKind::Image => "img",
            DomElementKind::Checkbox
            | DomElementKind::Range
            | DomElementKind::TextInput
            | DomElementKind::Stepper
            | DomElementKind::DateInput => "input",
            DomElementKind::Progress => "progress",
            DomElementKind::TextArea => "textarea",
            DomElementKind::Video => "video",
            DomElementKind::Iframe => "iframe",
            DomElementKind::Canvas => "canvas",
        }
    }

    /// Input type for element kinds represented by `<input>`.
    pub const fn input_type(self) -> Option<&'static str> {
        match self {
            DomElementKind::Checkbox => Some("checkbox"),
            DomElementKind::Range => Some("range"),
            DomElementKind::TextInput => Some("text"),
            DomElementKind::Stepper => Some("number"),
            DomElementKind::DateInput => Some("datetime-local"),
            _ => None,
        }
    }
}

/// A command for the wasm host layer to apply to the browser DOM.
#[derive(Debug, Clone, PartialEq)]
pub enum DomCommand {
    /// Create a DOM element.
    Create {
        /// Stable widget id.
        id: u64,
        /// Element kind.
        kind: DomElementKind,
    },
    /// Set a platform-neutral attribute on an element.
    SetAttribute {
        /// Stable widget id.
        id: u64,
        /// Attribute payload.
        attr: Attribute,
    },
    /// Apply absolute layout coordinates as CSS pixels.
    SetFrame {
        /// Stable widget id.
        id: u64,
        /// Left coordinate.
        x: f32,
        /// Top coordinate.
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
    },
    /// Insert a child into a parent element.
    InsertChild {
        /// Parent widget id.
        parent: u64,
        /// Child widget id.
        child: u64,
        /// Child index.
        index: usize,
    },
    /// Remove a child from a parent element.
    RemoveChild {
        /// Parent widget id.
        parent: u64,
        /// Child widget id.
        child: u64,
    },
    /// Remove an element from the DOM and free its host state.
    Destroy {
        /// Stable widget id.
        id: u64,
    },
    /// Attach the root element to the web mount node.
    SetRoot {
        /// Root widget id.
        id: u64,
    },
    /// Register a DOM event listener for a gesture.
    AddGesture {
        /// Stable widget id.
        id: u64,
        /// Gesture kind.
        gesture: GestureKind,
    },
    /// Set scrollable content dimensions.
    SetContentSize {
        /// Scroll widget id.
        id: u64,
        /// Content width.
        width: f32,
        /// Content height.
        height: f32,
    },
    /// Set document/body backdrop color.
    SetBackdrop {
        /// CSS color string.
        css_color: String,
    },
    /// Request web haptic feedback when available.
    Haptic {
        /// Haptic style.
        style: HapticStyle,
    },
    /// Invoke a browser/platform service.
    Request(WebPlatformRequest),
    /// Scroll an element to an explicit offset.
    ScrollTo {
        /// Scroll widget id.
        id: u64,
        /// Horizontal offset.
        offset_x: f32,
        /// Vertical offset.
        offset_y: f32,
        /// Whether the scroll should animate.
        animated: bool,
    },
    /// Scroll an element to the top-left origin.
    ScrollToTop {
        /// Scroll widget id.
        id: u64,
        /// Whether the scroll should animate.
        animated: bool,
    },
}

impl DomCommand {
    /// Translates a framework mutation into a DOM host command.
    pub fn from_mutation(mutation: Mutation) -> Self {
        match mutation {
            Mutation::Create { id, kind } => DomCommand::Create {
                id: widget_key(id),
                kind: DomElementKind::from_widget_kind(kind),
            },
            Mutation::SetAttribute { id, attr } => DomCommand::SetAttribute {
                id: widget_key(id),
                attr,
            },
            Mutation::SetFrame { id, rect } => DomCommand::SetFrame {
                id: widget_key(id),
                x: rect.origin.x,
                y: rect.origin.y,
                width: rect.size.width,
                height: rect.size.height,
            },
            Mutation::InsertChild {
                parent,
                index,
                child,
            } => DomCommand::InsertChild {
                parent: widget_key(parent),
                child: widget_key(child),
                index,
            },
            Mutation::RemoveChild { parent, child } => DomCommand::RemoveChild {
                parent: widget_key(parent),
                child: widget_key(child),
            },
            Mutation::Destroy { id } => DomCommand::Destroy { id: widget_key(id) },
            Mutation::SetRoot { id } => DomCommand::SetRoot { id: widget_key(id) },
            Mutation::AddGesture { id, gesture } => DomCommand::AddGesture {
                id: widget_key(id),
                gesture,
            },
            Mutation::SetContentSize { id, size } => DomCommand::SetContentSize {
                id: widget_key(id),
                width: size.width,
                height: size.height,
            },
            Mutation::SetBackdrop { color } => DomCommand::SetBackdrop {
                css_color: color_to_css(color),
            },
            Mutation::Haptic { style } => DomCommand::Haptic { style },
            Mutation::ScheduleNotification(notification) => {
                DomCommand::Request(WebPlatformRequest::ScheduleNotification(notification))
            }
            Mutation::CancelNotification { id } => {
                DomCommand::Request(WebPlatformRequest::CancelNotification { id })
            }
            Mutation::AuthenticateBiometric { reason } => {
                DomCommand::Request(WebPlatformRequest::AuthenticateBiometric { reason })
            }
            Mutation::StartLocation | Mutation::RequestLocation => {
                DomCommand::Request(WebPlatformRequest::StartLocation)
            }
            Mutation::StopLocation | Mutation::StopLocationUpdates => {
                DomCommand::Request(WebPlatformRequest::StopLocation)
            }
            Mutation::StartMotion {
                accelerometer,
                gyroscope,
            } => DomCommand::Request(WebPlatformRequest::StartMotion {
                accelerometer,
                gyroscope,
            }),
            Mutation::StopMotion => DomCommand::Request(WebPlatformRequest::StopMotion),
            Mutation::PresentMediaPicker { max_selection } => {
                DomCommand::Request(WebPlatformRequest::PresentMediaPicker { max_selection })
            }
            Mutation::PresentDocumentPicker { types } => {
                DomCommand::Request(WebPlatformRequest::PresentDocumentPicker { types })
            }
            Mutation::RegisterBackgroundTask { identifier } => {
                DomCommand::Request(WebPlatformRequest::RegisterBackgroundTask { identifier })
            }
            Mutation::ScheduleBackgroundTask {
                identifier,
                earliest_seconds,
            } => DomCommand::Request(WebPlatformRequest::ScheduleBackgroundTask {
                identifier,
                earliest_seconds,
            }),
            Mutation::SetClipboard { text } => {
                DomCommand::Request(WebPlatformRequest::SetClipboard { text })
            }
            Mutation::ShareText { text } => {
                DomCommand::Request(WebPlatformRequest::ShareText { text })
            }
            Mutation::AnnounceAccessibility { message } => {
                DomCommand::Request(WebPlatformRequest::AnnounceAccessibility { message })
            }
            Mutation::RequestFocus { id } => {
                DomCommand::Request(WebPlatformRequest::RequestFocus { id: widget_key(id) })
            }
            Mutation::SetTorch { on } => DomCommand::Request(WebPlatformRequest::SetTorch { on }),
            Mutation::RegisterForPushNotifications => {
                DomCommand::Request(WebPlatformRequest::RegisterForPushNotifications)
            }
            Mutation::SetAppBadge { count } => {
                DomCommand::Request(WebPlatformRequest::SetAppBadge { count })
            }
            Mutation::ScrollTo {
                id,
                offset_x,
                offset_y,
                animated,
            } => DomCommand::ScrollTo {
                id: widget_key(id),
                offset_x,
                offset_y,
                animated,
            },
            Mutation::ScrollToTop { id, animated } => DomCommand::ScrollToTop {
                id: widget_key(id),
                animated,
            },
        }
    }
}

/// Browser/platform-service work requested by app code.
#[derive(Debug, Clone, PartialEq)]
pub enum WebPlatformRequest {
    /// Schedule a web notification.
    ScheduleNotification(LocalNotification),
    /// Cancel a web notification.
    CancelNotification {
        /// Notification id.
        id: String,
    },
    /// Request WebAuthn or platform authentication.
    AuthenticateBiometric {
        /// Prompt reason.
        reason: String,
    },
    /// Start geolocation updates.
    StartLocation,
    /// Stop geolocation updates.
    StopLocation,
    /// Start browser motion sensor updates.
    StartMotion {
        /// Whether accelerometer updates are requested.
        accelerometer: bool,
        /// Whether gyroscope updates are requested.
        gyroscope: bool,
    },
    /// Stop motion sensor updates.
    StopMotion,
    /// Present a media picker.
    PresentMediaPicker {
        /// Maximum selection count.
        max_selection: usize,
    },
    /// Present a file picker.
    PresentDocumentPicker {
        /// Accepted MIME/type filters.
        types: Vec<String>,
    },
    /// Register background work.
    RegisterBackgroundTask {
        /// Task identifier.
        identifier: String,
    },
    /// Schedule background work.
    ScheduleBackgroundTask {
        /// Task identifier.
        identifier: String,
        /// Minimum delay before execution.
        earliest_seconds: f64,
    },
    /// Copy text to the clipboard.
    SetClipboard {
        /// Clipboard text.
        text: String,
    },
    /// Invoke the Web Share API.
    ShareText {
        /// Text to share.
        text: String,
    },
    /// Announce a screen-reader message.
    AnnounceAccessibility {
        /// Announcement text.
        message: String,
    },
    /// Move focus to an element.
    RequestFocus {
        /// Target widget id.
        id: u64,
    },
    /// Enable or disable torch where media APIs expose it.
    SetTorch {
        /// Whether the torch should be on.
        on: bool,
    },
    /// Register for push notifications.
    RegisterForPushNotifications,
    /// Set a badge count where the Badging API exists.
    SetAppBadge {
        /// Badge count.
        count: u32,
    },
}

/// A backend that records DOM commands for a wasm host to drain.
pub struct WebDomBackend {
    commands: DomCommandQueue,
}

impl Default for WebDomBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WebDomBackend {
    /// Creates an empty DOM backend command queue.
    pub fn new() -> Self {
        WebDomBackend {
            commands: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Returns a shared handle to the pending command queue.
    pub fn commands(&self) -> DomCommandQueue {
        self.commands.clone()
    }

    /// Drains pending commands from this backend.
    pub fn drain_commands(&self) -> Vec<DomCommand> {
        std::mem::take(&mut *self.commands.borrow_mut())
    }
}

impl Backend for WebDomBackend {
    fn apply(&mut self, mutation: Mutation) {
        self.commands
            .borrow_mut()
            .push(DomCommand::from_mutation(mutation));
    }
}

/// A running web app plus its DOM command queue.
pub struct WebDriver {
    app: App,
    commands: DomCommandQueue,
}

impl WebDriver {
    /// Mounts an app using the DOM command backend.
    pub fn new<V: View>(viewport: Size, make_view: impl FnOnce() -> V) -> Self {
        let backend = WebDomBackend::new();
        let commands = backend.commands();
        let app = App::new(Host::new(backend), viewport, make_view);
        WebDriver { app, commands }
    }

    /// Returns the event sink used by browser callbacks.
    pub fn event_sink(&self) -> EventSink {
        self.app.event_sink()
    }

    /// Enqueues a browser event for delivery on the next tick.
    pub fn dispatch_event(&self, event: Event) {
        self.event_sink().dispatch(event);
    }

    /// Advances one frame.
    pub fn tick(&mut self) {
        self.app.tick();
    }

    /// Updates the browser viewport.
    pub fn set_viewport(&mut self, viewport: Size) {
        self.app.set_viewport(viewport);
    }

    /// Drains commands emitted since the previous drain.
    pub fn drain_commands(&self) -> Vec<DomCommand> {
        std::mem::take(&mut *self.commands.borrow_mut())
    }

    /// Returns mutable access to the underlying app for platform-specific state updates.
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }
}

/// Converts a color into an `rgba(r, g, b, a)` CSS string.
pub fn color_to_css(color: Color) -> String {
    let alpha = color.a as f32 / 255.0;
    format!("rgba({}, {}, {}, {:.3})", color.r, color.g, color.b, alpha)
}

/// Converts a widget id into the stable integer key used by DOM nodes.
pub fn widget_key(id: WidgetId) -> u64 {
    id.to_u64()
}

/// Creates a frame command for tests and host bootstrap code.
pub fn frame_command(id: WidgetId, rect: Rect) -> DomCommand {
    DomCommand::SetFrame {
        id: widget_key(id),
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Attribute, Mutation, WidgetKind};
    use crate::view::{button, column, text};

    #[test]
    fn maps_widget_kinds_to_dom_elements() {
        let slider = DomElementKind::from_widget_kind(WidgetKind::Slider);
        assert_eq!(slider.tag_name(), "input");
        assert_eq!(slider.input_type(), Some("range"));

        let text = DomElementKind::from_widget_kind(WidgetKind::Text);
        assert_eq!(text.tag_name(), "span");
        assert_eq!(text.input_type(), None);
    }

    #[test]
    fn converts_backdrop_to_css_color() {
        let command = DomCommand::from_mutation(Mutation::SetBackdrop {
            color: Color::rgba(10, 20, 30, 128),
        });

        assert_eq!(
            command,
            DomCommand::SetBackdrop {
                css_color: "rgba(10, 20, 30, 0.502)".to_string()
            }
        );
    }

    #[test]
    fn driver_emits_initial_dom_commands() {
        let driver = WebDriver::new(Size::new(320.0, 480.0), || {
            column((text("Hello"), button("Tap", || {})))
        });
        let commands = driver.drain_commands();

        assert!(commands.iter().any(|command| matches!(
            command,
            DomCommand::Create {
                kind: DomElementKind::Span,
                ..
            }
        )));
        assert!(commands.iter().any(|command| matches!(
            command,
            DomCommand::SetAttribute {
                attr: Attribute::Text(value),
                ..
            } if value == "Hello"
        )));
        assert!(commands
            .iter()
            .any(|command| matches!(command, DomCommand::SetRoot { .. })));
    }
}
