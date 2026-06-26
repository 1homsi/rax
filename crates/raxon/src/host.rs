//! Shared host-session lifecycle for generated platform glue.
//!
//! Android JNI bindings, browser JavaScript shims, and future desktop hosts all
//! need to drive the same loop: deliver host events, advance the app, and drain
//! platform commands. This module keeps that orchestration in one place so each
//! platform backend only supplies its command encoder and event dispatcher.

#![forbid(unsafe_code)]

use std::fmt;

use crate::core::Size;
use crate::wire::WireProtocolError;

/// Error returned by host-session JSON entry points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostSessionError {
    /// A host-originated event batch could not be decoded or was unsupported.
    Event(WireProtocolError),
    /// A platform command batch could not be encoded.
    CommandJson(String),
}

impl fmt::Display for HostSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostSessionError::Event(error) => write!(f, "{error}"),
            HostSessionError::CommandJson(message) => {
                write!(f, "failed to encode host command batch: {message}")
            }
        }
    }
}

impl std::error::Error for HostSessionError {}

impl From<WireProtocolError> for HostSessionError {
    fn from(error: WireProtocolError) -> Self {
        HostSessionError::Event(error)
    }
}

impl From<serde_json::Error> for HostSessionError {
    fn from(error: serde_json::Error) -> Self {
        HostSessionError::CommandJson(error.to_string())
    }
}

/// A platform driver that can be advanced through a shared host transport loop.
pub trait HostDriver {
    /// Advances the app by one frame.
    fn tick(&mut self);

    /// Updates the host viewport in logical pixels.
    fn set_viewport(&mut self, viewport: Size);

    /// Decodes and enqueues a versioned JSON event batch from the host.
    fn dispatch_wire_event_batch_json(&self, payload: &str) -> Result<(), WireProtocolError>;

    /// Drains pending platform commands as a host-facing JSON batch.
    fn drain_command_batch_json(&self) -> Result<String, serde_json::Error>;
}

/// A running app session owned by generated platform glue.
pub struct HostSession<D> {
    driver: D,
}

impl<D> HostSession<D> {
    /// Creates a host session around a platform-specific driver.
    pub fn new(driver: D) -> Self {
        HostSession { driver }
    }

    /// Returns shared access to the platform driver.
    pub fn driver(&self) -> &D {
        &self.driver
    }

    /// Returns mutable access to the platform driver.
    pub fn driver_mut(&mut self) -> &mut D {
        &mut self.driver
    }

    /// Consumes the session and returns the wrapped platform driver.
    pub fn into_driver(self) -> D {
        self.driver
    }
}

impl<D: HostDriver> HostSession<D> {
    /// Advances one frame without draining commands.
    pub fn tick(&mut self) {
        self.driver.tick();
    }

    /// Updates the viewport in logical pixels.
    pub fn set_viewport(&mut self, viewport: Size) {
        self.driver.set_viewport(viewport);
    }

    /// Updates the viewport from raw logical dimensions.
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.set_viewport(Size::new(width, height));
    }

    /// Enqueues one decoded host event batch for delivery on the next tick.
    pub fn dispatch_event_batch_json(&self, payload: &str) -> Result<(), WireProtocolError> {
        self.driver.dispatch_wire_event_batch_json(payload)
    }

    /// Drains pending commands without advancing a frame.
    pub fn drain_command_batch_json(&self) -> Result<String, serde_json::Error> {
        self.driver.drain_command_batch_json()
    }

    /// Advances one frame, then drains the resulting platform command batch.
    pub fn tick_and_drain_command_batch_json(&mut self) -> Result<String, serde_json::Error> {
        self.tick();
        self.drain_command_batch_json()
    }

    /// Delivers host events, advances one frame, and drains platform commands.
    pub fn dispatch_events_tick_and_drain_command_batch_json(
        &mut self,
        payload: &str,
    ) -> Result<String, HostSessionError> {
        self.dispatch_event_batch_json(payload)?;
        Ok(self.tick_and_drain_command_batch_json()?)
    }

    /// Resizes the viewport, advances one frame, and drains platform commands.
    pub fn resize_tick_and_drain_command_batch_json(
        &mut self,
        width: f32,
        height: f32,
    ) -> Result<String, serde_json::Error> {
        self.set_viewport_size(width, height);
        self.tick_and_drain_command_batch_json()
    }
}
