# State & Reactivity

Our core advantage over RN/Flutter: fine-grained signals instead of a re-render
diff. Match the ergonomics of Redux/Zustand/Recoil/Riverpod/Provider while being
surgical. ✅ · 🟡 · ⬜.

## Primitives
- ✅ signals (sources), memos (derived), effects (sinks)
- ✅ glitch-free propagation, batching, untracked reads
- ✅ explicit `Runtime` + ownership scopes (auto-dispose)
- ✅ stores (struct-of-signals) + selectors (`Store<S>` — wraps `Signal<S>`, `store.select(|s| s.field)` → `Memo`, `store.update(|s| …)`)
- ✅ context / providers (dependency injection down the tree)
- ✅ `Resource` (async-aware signal: loading/error/data)
- ⬜ derived collections (fine-grained list reactivity, `keyed_for`)
- ⬜ writable computed / two-way bindings
- ⬜ signal equality customization / structural memo

## App-state patterns
- ✅ global stores + scoped stores (`Store<S>` is `Copy` — pass it anywhere; `provide_context(store)` for scoped)
- ✅ actions/reducers pattern (`Reducer` trait, `ReducerStore<S>{get/dispatch/signal}`, `use_reducer(initial)` — opt-in Elm/Redux style on top of signals)
- ✅ middleware / interceptors (`add_signal_middleware(fn(type_name, value))` — `notify_middlewares` called on set/update; `clear_signal_middlewares`)
- ✅ selectors with memoization (`store.select(fn) -> Memo<U>` — glitch-free derived memos)
- ✅ transactions / batched commits (`transaction(|| { ... })` alias for `batch` — exported for discoverability)
- ✅ optimistic updates + rollback (`store.optimistic_update(mutate_fn) -> rollback_fn` — snapshots state, applies mutation, returns closure to revert)

## Async & concurrency
- ⬜ suspense / transitions (pending UI without tearing)
- ✅ async derivations (`AsyncState<T>{Loading|Ready(T)|Error(String)}`, `create_async_derived(fut_fn)`, `create_deferred<T>() -> (Signal<AsyncState<T>>, resolve_fn)`); ⬜ debounce/throttle helpers
- ⬜ cross-thread signal writes marshaled to the UI thread (scheduler)
- ⬜ cancellation tied to ownership scopes

## Persistence & time-travel
- ✅ persisted signals/stores (`persisted_signal/bool/i64/f64(key, default)` — in-memory KV + `create_effect` write-through; bridge to UserDefaults via `kv_set/kv_get`)
- ⬜ hydration (SSR/web), state restoration (mobile)
- ⬜ time-travel debugging via devtools
- ✅ undo/redo helpers (`History<T>` — `use_history(init)`, `.push(val)`, `.undo()`, `.redo()`, `.can_undo/can_redo()`, `.signal()`)

## Tooling
- ⬜ signal-graph inspector (dependencies, recompute counts)
- ⬜ leak detection in CI
- ⬜ lints for common reactivity mistakes
