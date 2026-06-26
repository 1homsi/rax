//! Android backend foundation.
//!
//! This module provides the pure-Rust half of the Android backend: a command
//! queue that translates raxon [`Mutation`](crate::dom::Mutation)s into Android
//! view operations. JNI glue can drain these commands from an Activity and
//! apply them to real `android.view.View` instances while sending native events
//! back through [`EventSink`](crate::dom::EventSink).

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

/// A shared queue of Android commands produced by [`AndroidBackend`].
pub type AndroidCommandQueue = Rc<RefCell<Vec<AndroidCommand>>>;

/// Android view classes used by the first native backend pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidViewClass {
    /// A generic `FrameLayout` container.
    FrameLayout,
    /// A `TextView`.
    TextView,
    /// A `Button`.
    Button,
    /// An `ImageView`.
    ImageView,
    /// A `Switch`.
    Switch,
    /// A `SeekBar`.
    SeekBar,
    /// An `EditText`.
    EditText,
    /// A `ScrollView` / `HorizontalScrollView`.
    ScrollView,
    /// A `ProgressBar` in indeterminate mode.
    ActivityIndicator,
    /// A determinate `ProgressBar`.
    ProgressBar,
    /// A segmented-control host, typically backed by Material buttons.
    SegmentedControl,
    /// A numeric stepper host.
    Stepper,
    /// A `DatePicker` or `TimePicker` host.
    DatePicker,
    /// A camera preview host.
    CameraPreview,
    /// A `WebView`.
    WebView,
    /// A `RecyclerView`.
    RecyclerView,
    /// A map view host.
    MapView,
    /// A custom canvas view.
    CanvasView,
}

impl AndroidViewClass {
    /// Maps a framework widget kind to the Android view class that should back it.
    pub const fn from_widget_kind(kind: WidgetKind) -> Self {
        match kind {
            WidgetKind::View | WidgetKind::Stack => AndroidViewClass::FrameLayout,
            WidgetKind::Text => AndroidViewClass::TextView,
            WidgetKind::Button => AndroidViewClass::Button,
            WidgetKind::Image => AndroidViewClass::ImageView,
            WidgetKind::Switch => AndroidViewClass::Switch,
            WidgetKind::Slider => AndroidViewClass::SeekBar,
            WidgetKind::TextInput | WidgetKind::TextArea => AndroidViewClass::EditText,
            WidgetKind::Scroll => AndroidViewClass::ScrollView,
            WidgetKind::ActivityIndicator => AndroidViewClass::ActivityIndicator,
            WidgetKind::Progress => AndroidViewClass::ProgressBar,
            WidgetKind::Segmented => AndroidViewClass::SegmentedControl,
            WidgetKind::Stepper => AndroidViewClass::Stepper,
            WidgetKind::DatePicker => AndroidViewClass::DatePicker,
            WidgetKind::Camera => AndroidViewClass::CameraPreview,
            WidgetKind::WebView => AndroidViewClass::WebView,
            WidgetKind::LazyList => AndroidViewClass::RecyclerView,
            WidgetKind::MapView => AndroidViewClass::MapView,
            WidgetKind::Canvas => AndroidViewClass::CanvasView,
        }
    }

    /// Fully-qualified Android class name for the default host implementation.
    pub const fn class_name(self) -> &'static str {
        match self {
            AndroidViewClass::FrameLayout => "android.widget.FrameLayout",
            AndroidViewClass::TextView => "android.widget.TextView",
            AndroidViewClass::Button => "android.widget.Button",
            AndroidViewClass::ImageView => "android.widget.ImageView",
            AndroidViewClass::Switch => "android.widget.Switch",
            AndroidViewClass::SeekBar => "android.widget.SeekBar",
            AndroidViewClass::EditText => "android.widget.EditText",
            AndroidViewClass::ScrollView => "android.widget.ScrollView",
            AndroidViewClass::ActivityIndicator => "android.widget.ProgressBar",
            AndroidViewClass::ProgressBar => "android.widget.ProgressBar",
            AndroidViewClass::SegmentedControl => {
                "com.google.android.material.button.MaterialButtonToggleGroup"
            }
            AndroidViewClass::Stepper => "android.widget.NumberPicker",
            AndroidViewClass::DatePicker => "android.widget.DatePicker",
            AndroidViewClass::CameraPreview => "android.view.TextureView",
            AndroidViewClass::WebView => "android.webkit.WebView",
            AndroidViewClass::RecyclerView => "androidx.recyclerview.widget.RecyclerView",
            AndroidViewClass::MapView => "com.google.android.gms.maps.MapView",
            AndroidViewClass::CanvasView => "android.view.View",
        }
    }
}

/// A command for the Android host layer to apply to real views.
#[derive(Debug, Clone, PartialEq)]
pub enum AndroidCommand {
    /// Create a native view.
    Create {
        /// Stable widget id.
        id: u64,
        /// Android view class to instantiate.
        class: AndroidViewClass,
    },
    /// Set a platform-neutral attribute on a view.
    SetAttribute {
        /// Stable widget id.
        id: u64,
        /// Attribute payload.
        attr: Attribute,
    },
    /// Set a view frame in logical pixels.
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
    /// Insert a child into a parent container.
    InsertChild {
        /// Parent widget id.
        parent: u64,
        /// Child widget id.
        child: u64,
        /// Child index.
        index: usize,
    },
    /// Remove a child from a parent container.
    RemoveChild {
        /// Parent widget id.
        parent: u64,
        /// Child widget id.
        child: u64,
    },
    /// Destroy a native view.
    Destroy {
        /// Stable widget id.
        id: u64,
    },
    /// Attach the root view to the Activity content view.
    SetRoot {
        /// Root widget id.
        id: u64,
    },
    /// Register a gesture recognizer/listener.
    AddGesture {
        /// Stable widget id.
        id: u64,
        /// Gesture kind.
        gesture: GestureKind,
    },
    /// Set a scrollable content size.
    SetContentSize {
        /// Scroll widget id.
        id: u64,
        /// Content width.
        width: f32,
        /// Content height.
        height: f32,
    },
    /// Set the Activity/window backdrop color as Android `0xAARRGGBB`.
    SetBackdrop {
        /// Packed Android color.
        argb: u32,
    },
    /// Trigger Android haptic feedback.
    Haptic {
        /// Haptic style.
        style: HapticStyle,
    },
    /// Invoke a platform service.
    Request(AndroidPlatformRequest),
    /// Scroll a scroll view to an explicit offset.
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
    /// Scroll a scroll view to the top-left origin.
    ScrollToTop {
        /// Scroll widget id.
        id: u64,
        /// Whether the scroll should animate.
        animated: bool,
    },
}

impl AndroidCommand {
    /// Translates a framework mutation into an Android host command.
    pub fn from_mutation(mutation: Mutation) -> Self {
        match mutation {
            Mutation::Create { id, kind } => AndroidCommand::Create {
                id: widget_key(id),
                class: AndroidViewClass::from_widget_kind(kind),
            },
            Mutation::SetAttribute { id, attr } => AndroidCommand::SetAttribute {
                id: widget_key(id),
                attr,
            },
            Mutation::SetFrame { id, rect } => AndroidCommand::SetFrame {
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
            } => AndroidCommand::InsertChild {
                parent: widget_key(parent),
                child: widget_key(child),
                index,
            },
            Mutation::RemoveChild { parent, child } => AndroidCommand::RemoveChild {
                parent: widget_key(parent),
                child: widget_key(child),
            },
            Mutation::Destroy { id } => AndroidCommand::Destroy { id: widget_key(id) },
            Mutation::SetRoot { id } => AndroidCommand::SetRoot { id: widget_key(id) },
            Mutation::AddGesture { id, gesture } => AndroidCommand::AddGesture {
                id: widget_key(id),
                gesture,
            },
            Mutation::SetContentSize { id, size } => AndroidCommand::SetContentSize {
                id: widget_key(id),
                width: size.width,
                height: size.height,
            },
            Mutation::SetBackdrop { color } => AndroidCommand::SetBackdrop {
                argb: color.to_argb_u32(),
            },
            Mutation::Haptic { style } => AndroidCommand::Haptic { style },
            Mutation::ScheduleNotification(notification) => {
                AndroidCommand::Request(AndroidPlatformRequest::ScheduleNotification(notification))
            }
            Mutation::CancelNotification { id } => {
                AndroidCommand::Request(AndroidPlatformRequest::CancelNotification { id })
            }
            Mutation::AuthenticateBiometric { reason } => {
                AndroidCommand::Request(AndroidPlatformRequest::AuthenticateBiometric { reason })
            }
            Mutation::StartLocation | Mutation::RequestLocation => {
                AndroidCommand::Request(AndroidPlatformRequest::StartLocation)
            }
            Mutation::StopLocation | Mutation::StopLocationUpdates => {
                AndroidCommand::Request(AndroidPlatformRequest::StopLocation)
            }
            Mutation::StartMotion {
                accelerometer,
                gyroscope,
            } => AndroidCommand::Request(AndroidPlatformRequest::StartMotion {
                accelerometer,
                gyroscope,
            }),
            Mutation::StopMotion => AndroidCommand::Request(AndroidPlatformRequest::StopMotion),
            Mutation::PresentMediaPicker { max_selection } => {
                AndroidCommand::Request(AndroidPlatformRequest::PresentMediaPicker {
                    max_selection,
                })
            }
            Mutation::PresentDocumentPicker { types } => {
                AndroidCommand::Request(AndroidPlatformRequest::PresentDocumentPicker { types })
            }
            Mutation::RegisterBackgroundTask { identifier } => {
                AndroidCommand::Request(AndroidPlatformRequest::RegisterBackgroundTask {
                    identifier,
                })
            }
            Mutation::ScheduleBackgroundTask {
                identifier,
                earliest_seconds,
            } => AndroidCommand::Request(AndroidPlatformRequest::ScheduleBackgroundTask {
                identifier,
                earliest_seconds,
            }),
            Mutation::SetClipboard { text } => {
                AndroidCommand::Request(AndroidPlatformRequest::SetClipboard { text })
            }
            Mutation::ShareText { text } => {
                AndroidCommand::Request(AndroidPlatformRequest::ShareText { text })
            }
            Mutation::AnnounceAccessibility { message } => {
                AndroidCommand::Request(AndroidPlatformRequest::AnnounceAccessibility { message })
            }
            Mutation::RequestFocus { id } => {
                AndroidCommand::Request(AndroidPlatformRequest::RequestFocus { id: widget_key(id) })
            }
            Mutation::SetTorch { on } => {
                AndroidCommand::Request(AndroidPlatformRequest::SetTorch { on })
            }
            Mutation::RegisterForPushNotifications => {
                AndroidCommand::Request(AndroidPlatformRequest::RegisterForPushNotifications)
            }
            Mutation::SetAppBadge { count } => {
                AndroidCommand::Request(AndroidPlatformRequest::SetAppBadge { count })
            }
            Mutation::ScrollTo {
                id,
                offset_x,
                offset_y,
                animated,
            } => AndroidCommand::ScrollTo {
                id: widget_key(id),
                offset_x,
                offset_y,
                animated,
            },
            Mutation::ScrollToTop { id, animated } => AndroidCommand::ScrollToTop {
                id: widget_key(id),
                animated,
            },
        }
    }
}

/// Android platform-service work requested by app code.
#[derive(Debug, Clone, PartialEq)]
pub enum AndroidPlatformRequest {
    /// Schedule a local notification.
    ScheduleNotification(LocalNotification),
    /// Cancel a local notification.
    CancelNotification {
        /// Notification id.
        id: String,
    },
    /// Show a biometric prompt.
    AuthenticateBiometric {
        /// Prompt reason.
        reason: String,
    },
    /// Start location updates.
    StartLocation,
    /// Stop location updates.
    StopLocation,
    /// Start motion sensor updates.
    StartMotion {
        /// Whether accelerometer updates are requested.
        accelerometer: bool,
        /// Whether gyroscope updates are requested.
        gyroscope: bool,
    },
    /// Stop motion sensor updates.
    StopMotion,
    /// Present the Android media picker.
    PresentMediaPicker {
        /// Maximum selection count.
        max_selection: usize,
    },
    /// Present the Android document picker.
    PresentDocumentPicker {
        /// MIME or platform type filters.
        types: Vec<String>,
    },
    /// Register a background task name.
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
    /// Present a share sheet.
    ShareText {
        /// Text to share.
        text: String,
    },
    /// Announce a screen-reader message.
    AnnounceAccessibility {
        /// Announcement text.
        message: String,
    },
    /// Move accessibility/input focus.
    RequestFocus {
        /// Target widget id.
        id: u64,
    },
    /// Enable or disable the camera torch.
    SetTorch {
        /// Whether the torch should be on.
        on: bool,
    },
    /// Register for push notifications.
    RegisterForPushNotifications,
    /// Set the app badge count.
    SetAppBadge {
        /// Badge count.
        count: u32,
    },
}

/// A backend that records Android host commands for JNI glue to drain.
pub struct AndroidBackend {
    commands: AndroidCommandQueue,
}

impl Default for AndroidBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AndroidBackend {
    /// Creates an empty Android backend command queue.
    pub fn new() -> Self {
        AndroidBackend {
            commands: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Returns a shared handle to the pending command queue.
    pub fn commands(&self) -> AndroidCommandQueue {
        self.commands.clone()
    }

    /// Drains pending commands from this backend.
    pub fn drain_commands(&self) -> Vec<AndroidCommand> {
        std::mem::take(&mut *self.commands.borrow_mut())
    }
}

impl Backend for AndroidBackend {
    fn apply(&mut self, mutation: Mutation) {
        self.commands
            .borrow_mut()
            .push(AndroidCommand::from_mutation(mutation));
    }
}

/// A running Android app plus its command queue.
pub struct AndroidDriver {
    app: App,
    commands: AndroidCommandQueue,
}

impl AndroidDriver {
    /// Mounts an app using the Android command backend.
    pub fn new<V: View>(viewport: Size, make_view: impl FnOnce() -> V) -> Self {
        let backend = AndroidBackend::new();
        let commands = backend.commands();
        let app = App::new(Host::new(backend), viewport, make_view);
        AndroidDriver { app, commands }
    }

    /// Returns the event sink used by JNI callbacks.
    pub fn event_sink(&self) -> EventSink {
        self.app.event_sink()
    }

    /// Enqueues a native event for delivery on the next tick.
    pub fn dispatch_event(&self, event: Event) {
        self.event_sink().dispatch(event);
    }

    /// Advances one frame.
    pub fn tick(&mut self) {
        self.app.tick();
    }

    /// Updates the Activity viewport.
    pub fn set_viewport(&mut self, viewport: Size) {
        self.app.set_viewport(viewport);
    }

    /// Drains commands emitted since the previous drain.
    pub fn drain_commands(&self) -> Vec<AndroidCommand> {
        std::mem::take(&mut *self.commands.borrow_mut())
    }

    /// Returns mutable access to the underlying app for platform-specific state updates.
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }
}

/// Converts a color into Android's packed `0xAARRGGBB` layout.
pub const fn color_to_argb(color: Color) -> u32 {
    color.to_argb_u32()
}

/// Converts a widget id into the stable integer key used by Android views.
pub fn widget_key(id: WidgetId) -> u64 {
    id.to_u64()
}

/// Creates a zero-sized frame command for tests and host bootstrap code.
pub fn frame_command(id: WidgetId, rect: Rect) -> AndroidCommand {
    AndroidCommand::SetFrame {
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
    fn maps_widget_kinds_to_android_classes() {
        assert_eq!(
            AndroidViewClass::from_widget_kind(WidgetKind::Text).class_name(),
            "android.widget.TextView"
        );
        assert_eq!(
            AndroidViewClass::from_widget_kind(WidgetKind::LazyList).class_name(),
            "androidx.recyclerview.widget.RecyclerView"
        );
    }

    #[test]
    fn converts_backdrop_to_android_argb() {
        let command = AndroidCommand::from_mutation(Mutation::SetBackdrop {
            color: Color::rgba(0x11, 0x22, 0x33, 0x44),
        });

        assert_eq!(command, AndroidCommand::SetBackdrop { argb: 0x4411_2233 });
    }

    #[test]
    fn driver_emits_initial_view_commands() {
        let driver = AndroidDriver::new(Size::new(320.0, 480.0), || {
            column((text("Hello"), button("Tap", || {})))
        });
        let commands = driver.drain_commands();

        assert!(commands.iter().any(|command| matches!(
            command,
            AndroidCommand::Create {
                class: AndroidViewClass::TextView,
                ..
            }
        )));
        assert!(commands.iter().any(|command| matches!(
            command,
            AndroidCommand::SetAttribute {
                attr: Attribute::Text(value),
                ..
            } if value == "Hello"
        )));
        assert!(commands
            .iter()
            .any(|command| matches!(command, AndroidCommand::SetRoot { .. })));
    }
}
