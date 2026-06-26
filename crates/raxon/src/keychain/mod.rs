//! Secure key-value storage for rax apps.
//!
//! On iOS this is backed by the system Keychain via `Security.framework`.
//! On all other platforms (host tests, macOS dev builds) it falls back to an
//! in-memory `HashMap` so the API compiles and behaves correctly everywhere.
//!
//! # Example
//! ```rust
//! use raxon::keychain::{set_secret, get_secret, delete_secret};
//!
//! set_secret("auth_token", "my-secret-value").unwrap();
//! let token = get_secret("auth_token").unwrap(); // Some("my-secret-value")
//! delete_secret("auth_token").unwrap();
//! ```

// ---------------------------------------------------------------------------
// iOS implementation — wraps Security.framework SecItem* APIs.
// ---------------------------------------------------------------------------

// The iOS path is Security.framework FFI, which is inherently unsafe.
#[cfg(target_os = "ios")]
#[allow(unsafe_code)]
mod ios_impl {
    use std::os::raw::{c_char, c_void};

    // Minimal CoreFoundation / Security types we need.
    type CFTypeRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFDataRef = *const c_void;
    type CFDictionaryRef = *const c_void;
    type CFMutableDictionaryRef = *mut c_void;
    type CFAllocatorRef = *const c_void;
    type CFIndex = isize;
    type OSStatus = i32;

    const K_CF_ALLOCATOR_DEFAULT: CFAllocatorRef = std::ptr::null();
    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    const ERR_SEC_ITEM_NOT_FOUND: OSStatus = -25300;

    extern "C" {
        fn CFStringCreateWithCString(
            alloc: CFAllocatorRef,
            c_str: *const c_char,
            encoding: u32,
        ) -> CFStringRef;
        fn CFDataCreate(alloc: CFAllocatorRef, bytes: *const u8, length: CFIndex) -> CFDataRef;
        fn CFDataGetBytePtr(the_data: CFDataRef) -> *const u8;
        fn CFDataGetLength(the_data: CFDataRef) -> CFIndex;
        fn CFRelease(cf: CFTypeRef);
        fn CFDictionaryCreateMutable(
            alloc: CFAllocatorRef,
            capacity: CFIndex,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> CFMutableDictionaryRef;
        fn CFDictionarySetValue(
            the_dict: CFMutableDictionaryRef,
            key: CFTypeRef,
            value: CFTypeRef,
        );

        // Security framework
        fn SecItemAdd(attributes: CFDictionaryRef, result: *mut CFTypeRef) -> OSStatus;
        fn SecItemCopyMatching(query: CFDictionaryRef, result: *mut CFTypeRef) -> OSStatus;
        fn SecItemDelete(query: CFDictionaryRef) -> OSStatus;

        // kSecClass, kSecAttrService, kSecAttrAccount, kSecValueData,
        // kSecReturnData, kSecMatchLimitOne, kSecMatchLimit constants
        // are CFStringRef globals exported from Security.framework.
        static kSecClass: CFStringRef;
        static kSecClassGenericPassword: CFStringRef;
        static kSecAttrService: CFStringRef;
        static kSecAttrAccount: CFStringRef;
        static kSecValueData: CFStringRef;
        static kSecReturnData: CFStringRef;
        static kSecMatchLimit: CFStringRef;
        static kSecMatchLimitOne: CFTypeRef;

        // CFBoolean
        static kCFBooleanTrue: CFTypeRef;
    }

    // Null callbacks — values are already CF-managed.
    extern "C" {
        static kCFTypeDictionaryKeyCallBacks: c_void;
        static kCFTypeDictionaryValueCallBacks: c_void;
    }

    const SERVICE: &str = "rax-app";

    unsafe fn cf_string(s: &str) -> CFStringRef {
        let c = std::ffi::CString::new(s).unwrap();
        CFStringCreateWithCString(K_CF_ALLOCATOR_DEFAULT, c.as_ptr(), K_CF_STRING_ENCODING_UTF8)
    }

    unsafe fn make_base_query(account: &str) -> CFMutableDictionaryRef {
        let dict = CFDictionaryCreateMutable(
            K_CF_ALLOCATOR_DEFAULT,
            0,
            &kCFTypeDictionaryKeyCallBacks as *const _,
            &kCFTypeDictionaryValueCallBacks as *const _,
        );
        let svc = cf_string(SERVICE);
        let acc = cf_string(account);
        CFDictionarySetValue(dict, kSecClass as CFTypeRef, kSecClassGenericPassword as CFTypeRef);
        CFDictionarySetValue(dict, kSecAttrService as CFTypeRef, svc as CFTypeRef);
        CFDictionarySetValue(dict, kSecAttrAccount as CFTypeRef, acc as CFTypeRef);
        CFRelease(svc);
        CFRelease(acc);
        dict
    }

    pub fn set_secret(key: &str, value: &str) -> Result<(), String> {
        // Delete any existing entry first (simplest upsert strategy).
        delete_secret(key).ok();
        unsafe {
            let dict = make_base_query(key);
            let bytes = value.as_bytes();
            let data = CFDataCreate(K_CF_ALLOCATOR_DEFAULT, bytes.as_ptr(), bytes.len() as CFIndex);
            CFDictionarySetValue(dict, kSecValueData as CFTypeRef, data as CFTypeRef);
            CFRelease(data);
            let status = SecItemAdd(dict as CFDictionaryRef, std::ptr::null_mut());
            CFRelease(dict as CFTypeRef);
            if status == 0 {
                Ok(())
            } else {
                Err(format!("SecItemAdd failed: {}", status))
            }
        }
    }

    pub fn get_secret(key: &str) -> Result<Option<String>, String> {
        unsafe {
            let dict = make_base_query(key);
            CFDictionarySetValue(dict, kSecReturnData as CFTypeRef, kCFBooleanTrue);
            CFDictionarySetValue(dict, kSecMatchLimit as CFTypeRef, kSecMatchLimitOne);
            let mut result: CFTypeRef = std::ptr::null();
            let status = SecItemCopyMatching(dict as CFDictionaryRef, &mut result);
            CFRelease(dict as CFTypeRef);
            if status == ERR_SEC_ITEM_NOT_FOUND {
                return Ok(None);
            }
            if status != 0 {
                return Err(format!("SecItemCopyMatching failed: {}", status));
            }
            if result.is_null() {
                return Ok(None);
            }
            let data = result as CFDataRef;
            let len = CFDataGetLength(data) as usize;
            let ptr = CFDataGetBytePtr(data);
            let bytes = std::slice::from_raw_parts(ptr, len);
            let s = String::from_utf8_lossy(bytes).into_owned();
            CFRelease(result);
            Ok(Some(s))
        }
    }

    pub fn delete_secret(key: &str) -> Result<(), String> {
        unsafe {
            let dict = make_base_query(key);
            let status = SecItemDelete(dict as CFDictionaryRef);
            CFRelease(dict as CFTypeRef);
            if status == 0 || status == ERR_SEC_ITEM_NOT_FOUND {
                Ok(())
            } else {
                Err(format!("SecItemDelete failed: {}", status))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Non-iOS fallback — in-memory HashMap (suitable for tests and macOS dev).
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "ios"))]
mod mem_impl {
    use std::collections::HashMap;
    use std::sync::Mutex;

    static STORE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

    fn with_store<R>(f: impl FnOnce(&mut HashMap<String, String>) -> R) -> R {
        let mut guard = STORE.lock().unwrap();
        let map = guard.get_or_insert_with(HashMap::new);
        f(map)
    }

    pub fn set_secret(key: &str, value: &str) -> Result<(), String> {
        with_store(|m| { m.insert(key.to_string(), value.to_string()); });
        Ok(())
    }

    pub fn get_secret(key: &str) -> Result<Option<String>, String> {
        Ok(with_store(|m| m.get(key).cloned()))
    }

    pub fn delete_secret(key: &str) -> Result<(), String> {
        with_store(|m| { m.remove(key); });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Public API — delegates to the platform-appropriate implementation.
// ---------------------------------------------------------------------------

/// Store a secret value under `key`. Overwrites any existing value.
///
/// On iOS this writes to the system Keychain. On other platforms this uses
/// an in-memory store (for testing / development only).
pub fn set_secret(key: impl AsRef<str>, value: impl AsRef<str>) -> Result<(), String> {
    #[cfg(target_os = "ios")]
    return ios_impl::set_secret(key.as_ref(), value.as_ref());
    #[cfg(not(target_os = "ios"))]
    return mem_impl::set_secret(key.as_ref(), value.as_ref());
}

/// Retrieve the secret stored under `key`. Returns `None` if not found.
pub fn get_secret(key: impl AsRef<str>) -> Result<Option<String>, String> {
    #[cfg(target_os = "ios")]
    return ios_impl::get_secret(key.as_ref());
    #[cfg(not(target_os = "ios"))]
    return mem_impl::get_secret(key.as_ref());
}

/// Delete the secret stored under `key`. No-op if `key` is not present.
pub fn delete_secret(key: impl AsRef<str>) -> Result<(), String> {
    #[cfg(target_os = "ios")]
    return ios_impl::delete_secret(key.as_ref());
    #[cfg(not(target_os = "ios"))]
    return mem_impl::delete_secret(key.as_ref());
}
