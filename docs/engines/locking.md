# Engine Mutation Locking

YuTool serializes managed-engine mutations per engine ID.

Mutating operations acquire an exclusive lock before changing managed state:

- install;
- activate;
- deactivate;
- remove.

Lock files live under:

```text
<yu-data>/state/locks/<engine-id>.lock
```

The lock file itself is persistent; the operating-system lock is held only while the mutation guard is alive.

YuTool uses Rust standard-library file locks. A second process attempting to mutate the same engine fails immediately instead of waiting indefinitely. Different engine IDs may be mutated concurrently.

Read-only operations such as `engine versions` do not take the exclusive mutation lock because active-state replacement and version quarantine are atomic at the filesystem boundary.

The CLI maps a busy mutation to the frozen `OUTPUT_CONFLICT` error code.