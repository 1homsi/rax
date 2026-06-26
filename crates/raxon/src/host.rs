//! Shared host-session lifecycle for generated platform glue.
//!
//! Android JNI bindings, browser JavaScript shims, and future desktop hosts all
//! need to drive the same loop: deliver host events, advance the app, and drain
//! platform commands. This module keeps that orchestration in one place so each
//! platform backend only supplies its command encoder and event dispatcher.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

use crate::core::Size;
use crate::wire::WireProtocolError;

/// Current JSON bridge protocol version for host-session requests.
pub const HOST_BRIDGE_PROTOCOL_VERSION: u32 = 1;

/// Error returned by host-session JSON entry points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostSessionError {
    /// A generated host binding sent malformed request JSON.
    RequestJson(String),
    /// A generated host binding used an unsupported bridge version.
    UnsupportedBridgeVersion {
        /// Version supported by this runtime.
        expected: u32,
        /// Version sent by the generated host binding.
        found: u32,
    },
    /// A generated host binding referenced a session handle that no longer exists.
    UnknownSession {
        /// Opaque host-session handle.
        handle: u64,
    },
    /// A host-originated event batch could not be decoded or was unsupported.
    Event(WireProtocolError),
    /// A platform command batch could not be encoded.
    CommandJson(String),
    /// A host bridge response could not be encoded.
    ResponseJson(String),
}

impl fmt::Display for HostSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostSessionError::RequestJson(message) => {
                write!(f, "invalid host request JSON: {message}")
            }
            HostSessionError::UnsupportedBridgeVersion { expected, found } => {
                write!(
                    f,
                    "unsupported host bridge protocol version {found}; expected {expected}"
                )
            }
            HostSessionError::UnknownSession { handle } => {
                write!(f, "unknown host session handle {handle}")
            }
            HostSessionError::Event(error) => write!(f, "{error}"),
            HostSessionError::CommandJson(message) => {
                write!(f, "failed to encode host command batch: {message}")
            }
            HostSessionError::ResponseJson(message) => {
                write!(f, "failed to encode host response: {message}")
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

/// Opaque session id passed through generated platform bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HostSessionHandle(u64);

impl HostSessionHandle {
    /// Creates a handle from a raw host-owned id.
    pub const fn from_raw(raw: u64) -> Self {
        HostSessionHandle(raw)
    }

    /// Returns the raw id suitable for JNI, wasm, or C ABI boundaries.
    pub const fn to_raw(self) -> u64 {
        self.0
    }
}

/// Owns host sessions behind stable opaque handles.
///
/// Generated JNI/JS glue can keep one registry per app process or wasm module,
/// hand `HostSessionHandle::to_raw()` across the platform boundary, and route
/// all subsequent resize/event/tick/drain calls back through this type.
pub struct HostSessionRegistry<D> {
    next_handle: u64,
    sessions: BTreeMap<HostSessionHandle, HostSession<D>>,
}

impl<D> Default for HostSessionRegistry<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D> HostSessionRegistry<D> {
    /// Creates an empty host-session registry.
    pub fn new() -> Self {
        HostSessionRegistry {
            next_handle: 1,
            sessions: BTreeMap::new(),
        }
    }

    /// Inserts an already-mounted session and returns its opaque handle.
    pub fn insert_session(&mut self, session: HostSession<D>) -> HostSessionHandle {
        let handle = self.allocate_handle();
        self.sessions.insert(handle, session);
        handle
    }

    /// Inserts a platform driver by wrapping it in a [`HostSession`].
    pub fn insert_driver(&mut self, driver: D) -> HostSessionHandle {
        self.insert_session(HostSession::new(driver))
    }

    /// Removes a session, returning it to Rust if the handle was valid.
    pub fn remove(&mut self, handle: HostSessionHandle) -> Option<HostSession<D>> {
        self.sessions.remove(&handle)
    }

    /// Returns whether `handle` names a live session.
    pub fn contains(&self, handle: HostSessionHandle) -> bool {
        self.sessions.contains_key(&handle)
    }

    /// Number of live sessions.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Whether no sessions are registered.
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Shared access to a live session.
    pub fn get(&self, handle: HostSessionHandle) -> Option<&HostSession<D>> {
        self.sessions.get(&handle)
    }

    /// Mutable access to a live session.
    pub fn get_mut(&mut self, handle: HostSessionHandle) -> Option<&mut HostSession<D>> {
        self.sessions.get_mut(&handle)
    }

    fn allocate_handle(&mut self) -> HostSessionHandle {
        loop {
            let raw = self.next_handle.max(1);
            self.next_handle = raw.checked_add(1).unwrap_or(1);
            let handle = HostSessionHandle(raw);
            if !self.sessions.contains_key(&handle) {
                return handle;
            }
        }
    }
}

impl<D: HostDriver> HostSessionRegistry<D> {
    /// Advances one session by a frame.
    pub fn tick(&mut self, handle: HostSessionHandle) -> Result<(), HostSessionError> {
        self.session_mut(handle)?.tick();
        Ok(())
    }

    /// Updates one session viewport in logical pixels.
    pub fn set_viewport_size(
        &mut self,
        handle: HostSessionHandle,
        width: f32,
        height: f32,
    ) -> Result<(), HostSessionError> {
        self.session_mut(handle)?.set_viewport_size(width, height);
        Ok(())
    }

    /// Enqueues a JSON event batch for one session.
    pub fn dispatch_event_batch_json(
        &self,
        handle: HostSessionHandle,
        payload: &str,
    ) -> Result<(), HostSessionError> {
        self.session(handle)?.dispatch_event_batch_json(payload)?;
        Ok(())
    }

    /// Drains pending command JSON for one session.
    pub fn drain_command_batch_json(
        &self,
        handle: HostSessionHandle,
    ) -> Result<String, HostSessionError> {
        Ok(self.session(handle)?.drain_command_batch_json()?)
    }

    /// Advances one session, then drains pending command JSON.
    pub fn tick_and_drain_command_batch_json(
        &mut self,
        handle: HostSessionHandle,
    ) -> Result<String, HostSessionError> {
        Ok(self
            .session_mut(handle)?
            .tick_and_drain_command_batch_json()?)
    }

    /// Delivers events, advances one frame, and drains command JSON for one session.
    pub fn dispatch_events_tick_and_drain_command_batch_json(
        &mut self,
        handle: HostSessionHandle,
        payload: &str,
    ) -> Result<String, HostSessionError> {
        self.session_mut(handle)?
            .dispatch_events_tick_and_drain_command_batch_json(payload)
    }

    /// Resizes one session, advances a frame, and drains command JSON.
    pub fn resize_tick_and_drain_command_batch_json(
        &mut self,
        handle: HostSessionHandle,
        width: f32,
        height: f32,
    ) -> Result<String, HostSessionError> {
        Ok(self
            .session_mut(handle)?
            .resize_tick_and_drain_command_batch_json(width, height)?)
    }

    /// Applies one decoded host bridge request.
    pub fn handle_request(
        &mut self,
        request: HostBridgeRequest,
    ) -> Result<HostBridgeResponse, HostSessionError> {
        match request {
            HostBridgeRequest::Destroy { handle } => {
                let handle = HostSessionHandle::from_raw(handle);
                let _ = self.session(handle)?;
                self.remove(handle);
                Ok(HostBridgeResponse::Destroyed {
                    handle: handle.to_raw(),
                })
            }
            HostBridgeRequest::SetViewport {
                handle,
                width,
                height,
            } => {
                self.set_viewport_size(HostSessionHandle::from_raw(handle), width, height)?;
                Ok(HostBridgeResponse::Ok)
            }
            HostBridgeRequest::DispatchEventBatch { handle, batch } => {
                let payload = batch
                    .encode_json()
                    .map_err(|error| HostSessionError::RequestJson(error.to_string()))?;
                self.dispatch_event_batch_json(HostSessionHandle::from_raw(handle), &payload)?;
                Ok(HostBridgeResponse::Ok)
            }
            HostBridgeRequest::DrainCommandBatch { handle } => {
                let json = self.drain_command_batch_json(HostSessionHandle::from_raw(handle))?;
                Ok(HostBridgeResponse::CommandBatch {
                    batch: command_batch_value(&json)?,
                })
            }
            HostBridgeRequest::TickAndDrainCommandBatch { handle } => {
                let json =
                    self.tick_and_drain_command_batch_json(HostSessionHandle::from_raw(handle))?;
                Ok(HostBridgeResponse::CommandBatch {
                    batch: command_batch_value(&json)?,
                })
            }
            HostBridgeRequest::DispatchEventsTickAndDrainCommandBatch { handle, batch } => {
                let payload = batch
                    .encode_json()
                    .map_err(|error| HostSessionError::RequestJson(error.to_string()))?;
                let json = self.dispatch_events_tick_and_drain_command_batch_json(
                    HostSessionHandle::from_raw(handle),
                    &payload,
                )?;
                Ok(HostBridgeResponse::CommandBatch {
                    batch: command_batch_value(&json)?,
                })
            }
            HostBridgeRequest::ResizeTickAndDrainCommandBatch {
                handle,
                width,
                height,
            } => {
                let json = self.resize_tick_and_drain_command_batch_json(
                    HostSessionHandle::from_raw(handle),
                    width,
                    height,
                )?;
                Ok(HostBridgeResponse::CommandBatch {
                    batch: command_batch_value(&json)?,
                })
            }
        }
    }

    /// Applies one JSON host bridge request and returns a JSON bridge response.
    pub fn handle_request_json(&mut self, payload: &str) -> Result<String, HostSessionError> {
        let request: HostBridgeJsonRequest = serde_json::from_str(payload)
            .map_err(|error| HostSessionError::RequestJson(error.to_string()))?;
        request.ensure_supported()?;
        let response = self.handle_request(request.request)?;
        serde_json::to_string(&HostBridgeJsonResponse::new(response))
            .map_err(|error| HostSessionError::ResponseJson(error.to_string()))
    }

    fn session(&self, handle: HostSessionHandle) -> Result<&HostSession<D>, HostSessionError> {
        self.sessions
            .get(&handle)
            .ok_or(HostSessionError::UnknownSession {
                handle: handle.to_raw(),
            })
    }

    fn session_mut(
        &mut self,
        handle: HostSessionHandle,
    ) -> Result<&mut HostSession<D>, HostSessionError> {
        self.sessions
            .get_mut(&handle)
            .ok_or(HostSessionError::UnknownSession {
                handle: handle.to_raw(),
            })
    }
}

/// Versioned JSON envelope for host-originated bridge requests.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostBridgeJsonRequest {
    /// Bridge protocol version used by the generated host binding.
    pub protocol_version: u32,
    /// Decoded host request payload.
    #[serde(flatten)]
    pub request: HostBridgeRequest,
}

impl HostBridgeJsonRequest {
    /// Creates a request envelope using the current bridge protocol version.
    pub const fn new(request: HostBridgeRequest) -> Self {
        HostBridgeJsonRequest {
            protocol_version: HOST_BRIDGE_PROTOCOL_VERSION,
            request,
        }
    }

    /// Ensures this envelope uses the bridge version supported by this runtime.
    pub fn ensure_supported(&self) -> Result<(), HostSessionError> {
        if self.protocol_version == HOST_BRIDGE_PROTOCOL_VERSION {
            Ok(())
        } else {
            Err(HostSessionError::UnsupportedBridgeVersion {
                expected: HOST_BRIDGE_PROTOCOL_VERSION,
                found: self.protocol_version,
            })
        }
    }
}

/// A host-originated lifecycle request routed through a session registry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostBridgeRequest {
    /// Remove a session from its registry.
    Destroy {
        /// Opaque session handle.
        handle: u64,
    },
    /// Set the viewport without ticking.
    SetViewport {
        /// Opaque session handle.
        handle: u64,
        /// Viewport width in logical pixels.
        width: f32,
        /// Viewport height in logical pixels.
        height: f32,
    },
    /// Dispatch host events without ticking.
    DispatchEventBatch {
        /// Opaque session handle.
        handle: u64,
        /// Versioned host event batch.
        batch: crate::wire::WireEventBatch,
    },
    /// Drain currently queued platform commands.
    DrainCommandBatch {
        /// Opaque session handle.
        handle: u64,
    },
    /// Tick a session and drain platform commands.
    TickAndDrainCommandBatch {
        /// Opaque session handle.
        handle: u64,
    },
    /// Dispatch events, tick, and drain platform commands.
    DispatchEventsTickAndDrainCommandBatch {
        /// Opaque session handle.
        handle: u64,
        /// Versioned host event batch.
        batch: crate::wire::WireEventBatch,
    },
    /// Resize, tick, and drain platform commands.
    ResizeTickAndDrainCommandBatch {
        /// Opaque session handle.
        handle: u64,
        /// Viewport width in logical pixels.
        width: f32,
        /// Viewport height in logical pixels.
        height: f32,
    },
}

/// Versioned JSON envelope for host bridge responses.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostBridgeJsonResponse {
    /// Bridge protocol version used by this runtime.
    pub protocol_version: u32,
    /// Decoded host response payload.
    #[serde(flatten)]
    pub response: HostBridgeResponse,
}

impl HostBridgeJsonResponse {
    /// Creates a response envelope using the current bridge protocol version.
    pub const fn new(response: HostBridgeResponse) -> Self {
        HostBridgeJsonResponse {
            protocol_version: HOST_BRIDGE_PROTOCOL_VERSION,
            response,
        }
    }
}

/// Host bridge response payload returned to generated platform glue.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostBridgeResponse {
    /// The request succeeded and produced no command batch.
    Ok,
    /// A session was destroyed.
    Destroyed {
        /// Opaque session handle that was removed.
        handle: u64,
    },
    /// A host-facing platform command batch.
    CommandBatch {
        /// Command batch JSON as a nested value, not an escaped string.
        batch: serde_json::Value,
    },
}

fn command_batch_value(json: &str) -> Result<serde_json::Value, HostSessionError> {
    serde_json::from_str(json).map_err(|error| HostSessionError::CommandJson(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    use serde_json::{json, Value};

    use crate::wire::{WireEvent, WireEventBatch};

    #[derive(Debug)]
    struct RecordingDriver {
        ticks: usize,
        viewport: Size,
        events: RefCell<Vec<WireEventBatch>>,
        command_batches: RefCell<Vec<Value>>,
    }

    impl RecordingDriver {
        fn new(command_batches: Vec<Value>) -> Self {
            RecordingDriver {
                ticks: 0,
                viewport: Size::ZERO,
                events: RefCell::new(Vec::new()),
                command_batches: RefCell::new(command_batches),
            }
        }
    }

    impl HostDriver for RecordingDriver {
        fn tick(&mut self) {
            self.ticks += 1;
        }

        fn set_viewport(&mut self, viewport: Size) {
            self.viewport = viewport;
        }

        fn dispatch_wire_event_batch_json(&self, payload: &str) -> Result<(), WireProtocolError> {
            self.events
                .borrow_mut()
                .push(WireEventBatch::decode_json(payload)?);
            Ok(())
        }

        fn drain_command_batch_json(&self) -> Result<String, serde_json::Error> {
            let mut batches = self.command_batches.borrow_mut();
            let batch = if batches.is_empty() {
                json!({ "commands": [] })
            } else {
                batches.remove(0)
            };
            serde_json::to_string(&batch)
        }
    }

    #[test]
    fn bridge_json_routes_resize_tick_and_nested_command_batch() {
        let mut registry = HostSessionRegistry::new();
        let handle = registry.insert_driver(RecordingDriver::new(vec![json!({
            "commands": [{ "kind": "mount", "id": 7 }]
        })]));

        let request = HostBridgeRequest::ResizeTickAndDrainCommandBatch {
            handle: handle.to_raw(),
            width: 375.0,
            height: 812.0,
        };
        let response = registry
            .handle_request_json(
                &serde_json::to_string(&HostBridgeJsonRequest::new(request))
                    .expect("request encodes"),
            )
            .expect("bridge request succeeds");

        assert_eq!(
            serde_json::from_str::<HostBridgeJsonResponse>(&response).expect("response decodes"),
            HostBridgeJsonResponse::new(HostBridgeResponse::CommandBatch {
                batch: json!({ "commands": [{ "kind": "mount", "id": 7 }] }),
            })
        );
        let driver = registry.get(handle).expect("session remains live").driver();
        assert_eq!(driver.ticks, 1);
        assert_eq!(driver.viewport, Size::new(375.0, 812.0));
    }

    #[test]
    fn bridge_json_dispatches_events_and_destroy_errors_after_removal() {
        let mut registry = HostSessionRegistry::new();
        let handle = registry.insert_driver(RecordingDriver::new(Vec::new()));
        let batch = WireEventBatch::new(vec![WireEvent::Tap { target: 42 }]);

        let response = registry
            .handle_request(HostBridgeRequest::DispatchEventBatch {
                handle: handle.to_raw(),
                batch,
            })
            .expect("event dispatch succeeds");
        assert_eq!(response, HostBridgeResponse::Ok);
        assert_eq!(
            registry
                .get(handle)
                .expect("session remains live")
                .driver()
                .events
                .borrow()[0]
                .events,
            vec![WireEvent::Tap { target: 42 }]
        );

        let response = registry
            .handle_request(HostBridgeRequest::Destroy {
                handle: handle.to_raw(),
            })
            .expect("destroy succeeds");
        assert_eq!(
            response,
            HostBridgeResponse::Destroyed {
                handle: handle.to_raw(),
            }
        );
        assert_eq!(
            registry.handle_request(HostBridgeRequest::TickAndDrainCommandBatch {
                handle: handle.to_raw(),
            }),
            Err(HostSessionError::UnknownSession {
                handle: handle.to_raw(),
            })
        );
    }

    #[test]
    fn bridge_json_reports_invalid_request_json_and_unknown_handles() {
        let mut registry: HostSessionRegistry<RecordingDriver> = HostSessionRegistry::new();
        assert!(matches!(
            registry.handle_request_json("{not valid json"),
            Err(HostSessionError::RequestJson(_))
        ));
        assert_eq!(
            registry.handle_request_json(
                &serde_json::to_string(&HostBridgeJsonRequest {
                    protocol_version: HOST_BRIDGE_PROTOCOL_VERSION + 1,
                    request: HostBridgeRequest::DrainCommandBatch { handle: 99 },
                })
                .expect("request encodes")
            ),
            Err(HostSessionError::UnsupportedBridgeVersion {
                expected: HOST_BRIDGE_PROTOCOL_VERSION,
                found: HOST_BRIDGE_PROTOCOL_VERSION + 1,
            })
        );
        assert_eq!(
            registry.handle_request(HostBridgeRequest::DrainCommandBatch { handle: 99 }),
            Err(HostSessionError::UnknownSession { handle: 99 })
        );
    }
}
