//! Plugin system for rax native modules.
//!
//! Implement the [`Plugin`] trait to create a native module that integrates
//! with the rax event system and lifecycle.
//!
//! # Example
//! ```rust
//! use rax_plugin::Plugin;
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &'static str { "my-plugin" }
//!     fn on_start(&mut self) { /* init hardware, register callbacks */ }
//!     fn on_stop(&mut self) { /* cleanup */ }
//! }
//! ```
//!
//! Register plugins via `rax_plugin::register_plugin(MyPlugin)` before calling `rax::run`.

use std::any::Any;

/// The core plugin trait. Implement this to create a native rax module.
pub trait Plugin: Any + Send + 'static {
    /// Unique name for this plugin (used for conflict detection).
    fn name(&self) -> &'static str;

    /// Called once when the rax runtime starts.
    fn on_start(&mut self) {}

    /// Called once when the rax runtime stops.
    fn on_stop(&mut self) {}

    /// Called every frame tick. Use sparingly — prefer event-driven callbacks.
    fn on_tick(&mut self) {}

    /// Called when the app transitions to the background (e.g. user switches away).
    /// Plugins should pause timers, flush caches, or release foreground resources.
    fn on_background(&mut self) {}

    /// Called when the app returns to the foreground (e.g. user switches back).
    /// Plugins should resume paused work and refresh stale state.
    fn on_foreground(&mut self) {}

    /// Handle a custom event string from the event bus.
    fn on_event(&mut self, _event: &str, _payload: &str) {}
}

/// Registry of active plugins.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        PluginRegistry { plugins: vec![] }
    }

    /// Register a plugin. Panics if a plugin with the same name is already registered.
    pub fn register(&mut self, plugin: impl Plugin) {
        let name = plugin.name();
        if self.plugins.iter().any(|p| p.name() == name) {
            panic!("Plugin '{}' is already registered", name);
        }
        self.plugins.push(Box::new(plugin));
    }

    /// Call `on_start` on all registered plugins.
    pub fn start_all(&mut self) {
        for p in &mut self.plugins {
            p.on_start();
        }
    }

    /// Call `on_tick` on all registered plugins.
    pub fn tick_all(&mut self) {
        for p in &mut self.plugins {
            p.on_tick();
        }
    }

    /// Call `on_stop` on all registered plugins.
    pub fn stop_all(&mut self) {
        for p in &mut self.plugins {
            p.on_stop();
        }
    }

    /// Call `on_background` on all registered plugins.
    pub fn background_all(&mut self) {
        for p in &mut self.plugins {
            p.on_background();
        }
    }

    /// Call `on_foreground` on all registered plugins.
    pub fn foreground_all(&mut self) {
        for p in &mut self.plugins {
            p.on_foreground();
        }
    }

    /// Dispatch a custom event to all plugins.
    pub fn dispatch(&mut self, event: &str, payload: &str) {
        for p in &mut self.plugins {
            p.on_event(event, payload);
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static PLUGIN_REGISTRY: std::cell::RefCell<PluginRegistry> =
        std::cell::RefCell::new(PluginRegistry::new());
}

/// Register a plugin with the global registry.
pub fn register_plugin(plugin: impl Plugin) {
    PLUGIN_REGISTRY.with(|r| r.borrow_mut().register(plugin));
}

/// Dispatch a custom event to all registered plugins.
pub fn dispatch_plugin_event(event: &str, payload: &str) {
    PLUGIN_REGISTRY.with(|r| r.borrow_mut().dispatch(event, payload));
}

/// Call `on_tick` on all plugins. Called by the rax runtime each frame.
pub fn tick_plugins() {
    PLUGIN_REGISTRY.with(|r| r.borrow_mut().tick_all());
}

/// Start all plugins. Called once by `rax::run` at startup.
pub fn start_plugins() {
    PLUGIN_REGISTRY.with(|r| r.borrow_mut().start_all());
}

/// Notify all registered plugins that the app went to the background.
/// Call this from the platform-specific app delegate / activity lifecycle.
pub fn background_plugins() {
    PLUGIN_REGISTRY.with(|r| r.borrow_mut().background_all());
}

/// Notify all registered plugins that the app returned to the foreground.
/// Call this from the platform-specific app delegate / activity lifecycle.
pub fn foreground_plugins() {
    PLUGIN_REGISTRY.with(|r| r.borrow_mut().foreground_all());
}
