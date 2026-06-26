//! `NSUserDefaults`-backed persistent storage for `rax-store`.
//!
//! Registered at app start so `crate::store::store_get/store_set` (and
//! `persisted` signals) survive relaunches, the way `localStorage` /
//! `AsyncStorage` does on the web / React Native.

use objc2::msg_send;
use objc2::rc::Retained;
use objc2_foundation::{NSString, NSUserDefaults};

use crate::store::Storage;

/// Storage backed by the standard user defaults.
pub(crate) struct UiKitStorage;

impl UiKitStorage {
    fn defaults() -> Retained<NSUserDefaults> {
        NSUserDefaults::standardUserDefaults()
    }
}

impl Storage for UiKitStorage {
    fn get(&self, key: &str) -> Option<String> {
        let k = NSString::from_str(key);
        let value: Option<Retained<NSString>> =
            unsafe { msg_send![&Self::defaults(), stringForKey: &*k] };
        value.map(|s| s.to_string())
    }

    fn set(&self, key: &str, value: &str) {
        let k = NSString::from_str(key);
        let v = NSString::from_str(value);
        unsafe {
            let _: () = msg_send![&Self::defaults(), setObject: &*v, forKey: &*k];
        }
    }

    fn remove(&self, key: &str) {
        let k = NSString::from_str(key);
        unsafe {
            let _: () = msg_send![&Self::defaults(), removeObjectForKey: &*k];
        }
    }
}
