use crate::reactive::create_root;

use super::{persisted, set_storage, store_get, store_set, MemoryStorage};

#[test]
fn kv_roundtrip() {
    set_storage(MemoryStorage::new());
    assert_eq!(store_get("k"), None);
    store_set("k", "v");
    assert_eq!(store_get("k").as_deref(), Some("v"));
}

#[test]
fn persisted_signal_seeds_default_and_writes_back() {
    set_storage(MemoryStorage::new());
    let (_, scope) = create_root(|| {
        let name = persisted("user.name", "Guest");
        assert_eq!(name.get(), "Guest");
        // The effect wrote the initial value through to storage.
        assert_eq!(store_get("user.name").as_deref(), Some("Guest"));

        name.set("Sam".to_string());
        assert_eq!(store_get("user.name").as_deref(), Some("Sam"));
    });
    scope.dispose();
}

#[test]
fn persisted_signal_loads_existing_value() {
    set_storage(MemoryStorage::new());
    store_set("counter", "41");
    let (value, scope) = create_root(|| persisted("counter", "0").get());
    assert_eq!(value, "41", "loaded from storage, ignoring the default");
    scope.dispose();
}
