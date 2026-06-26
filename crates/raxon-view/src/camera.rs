//! Camera preview + QR scanner view.

use raxon_dom::{Attribute, Event, EventKind, Tree, WidgetId};

use crate::view::View;

/// A camera preview view that fires a callback when a QR code is detected.
///
/// Internally creates a `WidgetKind::Camera` node backed by an
/// `AVCaptureSession` on iOS.  Use [`camera_scanner`] to construct one.
pub struct CameraScanner<F> {
    on_qr: F,
}

/// Creates a camera preview view that calls `on_qr` with the decoded string
/// whenever a QR code is detected in the frame.
///
/// ```rust,ignore
/// camera_scanner(move |qr| last_qr.set(qr))
///     .grow()
/// ```
pub fn camera_scanner<F: FnMut(String) + 'static>(on_qr: F) -> CameraScanner<F> {
    CameraScanner { on_qr }
}

impl<F: FnMut(String) + 'static> View for CameraScanner<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_camera();
        tree.set(id, Attribute::QrScanning(true));
        let mut on_qr = self.on_qr;
        tree.on(id, EventKind::QrDetected, move |event| {
            if let Event::QrDetected { value, .. } = event {
                on_qr(value.clone());
            }
        });
        id
    }
}
