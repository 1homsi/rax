# Storage & Persistence

Match RN AsyncStorage/MMKV/SQLite/MMKV/SecureStore and Flutter
shared_preferences/sqflite/hive/secure_storage. ⬜ planned.

## Key-value
- ✅ simple async KV store (prefs) — typed
- ⬜ fast synchronous KV (MMKV-style)
- ✅ namespaced / scoped stores (`KvNamespace::new(prefix)` — `.set/.get/.keys/.clear/.persisted()`; keys stored as `"<ns>.<key>"`)
- ✅ reactive storage (persisted signals)

## Structured / database
- ✅ SQLite (`raxon-sqlite::Database` — rusqlite bundled, open/execute/query/query_with)
- ✅ migrations (versioned, automatic — `Database::migrate(&[(version, sql)])` tracks applied versions in `_rax_migrations`)
- ✅ reactive SQLite queries (`use_reactive_query(initial) -> ReactiveQuery<T>`; `.invalidate()` bumps version signal; `.refresh(fetch_fn)` re-runs query; `.get()` returns current `Vec<T>`)
- ⬜ embedded document/KV DB option (sled/redb)
- ⬜ full-text search
- ✅ reactive KV queries (`watch_kv(key) -> Signal<Option<String>>`; `kv_set_reactive/kv_delete_reactive` for push notifications)

## Files & blobs
- ✅ file system access (`raxon-fs`: `app_documents_dir/cache_dir/temp_dir/support_dir`; `read_text/bytes`, `write_text/bytes`, `append_text`, `delete_file`, `list_files`, `exists`, `file_size`, `create_dir`)
- ⬜ streaming read/write, large files
- ⬜ blob/asset storage + image cache integration

## Secure & sensitive
- ✅ secure storage (`raxon-keychain`: `set_secret/get_secret/delete_secret` — Security.framework SecItem* on iOS; in-memory fallback on other platforms)
- ⬜ encryption at rest
- ⬜ biometric-gated storage

## Sync & lifecycle
- ⬜ state restoration (navigation + app state)
- ⬜ hydration for web/SSR
- ⬜ offline-first sync + conflict resolution
- ⬜ backup/restore, export/import
- ⬜ pluggable storage backends
- ⬜ storage inspector in devtools
