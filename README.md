# Bevy Observer Double-Free Crash Reproduction

Minimal reproduction of a double-free crash when using `observe()` with captured heap data in Bevy 0.17.x.

## Bug Summary

When spawning entities with `observe()` closures that capture heap-allocated data (e.g., `String`), then despawning those entities and spawning new ones with similar observers, a double-free memory corruption occurs.

### Affected Versions
- ❌ Bevy 0.17.0, 0.17.3 (crash confirmed)
- ✅ Bevy 0.18.0+ (fixed)

### Fix
The bug was fixed in [PR #19451](https://github.com/bevyengine/bevy/pull/19451) - "Improved Entity Lifecycle: remove flushing, support manual spawning and despawning".

This commit (`279a8678d`) is included in Bevy 0.18.0.

## Reproduction Steps

1. Clone and run:
   ```bash
   cargo run --release
   ```

2. Click anywhere in the window repeatedly (usually 2-4 clicks)

3. Observe crash:
   ```
   Segmentation fault (core dumped)
   ```
   or
   ```
   free(): double free detected in tcache 2
   ```

## Root Cause

The `observe()` function creates an observer entity that stores the closure. When the closure captures heap data (like a cloned `String`), that data is owned by the observer. During entity despawn, the observer cleanup doesn't properly manage ownership when entities are rapidly despawned and new ones spawned, leading to double-free of the captured data.

## Workaround (for Bevy 0.17.x)

Instead of capturing data in the closure, store it as a component on the entity and query it:

```rust
// BUGGY - captures String in closure:
commands.spawn((
    MyComponent,
    observe({
        let data = my_string.clone();
        move |_: On<Pointer<Click>>| {
            println!("{}", data);
        }
    }),
));

// WORKAROUND - store data on entity:
#[derive(Component)]
struct DataHolder(String);

commands.spawn((
    MyComponent,
    DataHolder(my_string.clone()),
    observe(|trigger: On<Pointer<Click>>, query: Query<&DataHolder>| {
        if let Ok(data) = query.get(trigger.entity) {
            println!("{}", data.0);
        }
    }),
));
```

## ASan Output

Running with AddressSanitizer shows:
```
ERROR: AddressSanitizer: attempting double-free on 0x...
```

The allocation and both free calls point to the same `String::clone()` in the observer closure capture.

## Environment

- Rust: nightly-2026-02-15
- OS: Linux
- Bevy: 0.17.3

## Related

- [Bevy Issue](https://github.com/bevyengine/bevy/issues/...) (if created)
- [Bevy PR #19451](https://github.com/bevyengine/bevy/pull/19451) - The fix
