## 1. Persistence Model and I/O

- [ ] 1.1 Add serializable persistence models for target-keyed history snapshots and define the working-directory history file name.
- [ ] 1.2 Implement history load flow that reads `current_dir()/.net-use-address-history.json` and falls back to empty cache on missing/invalid file.
- [ ] 1.3 Implement history save flow that writes cache snapshots through temp-file + rename replacement.

## 2. TUI Integration

- [ ] 2.1 Initialize `tui_main_loop` target cache from persisted history before entering monitor loops.
- [ ] 2.2 Keep restore-on-target-selection behavior using the loaded target-keyed cache.
- [ ] 2.3 Persist updated cache after each monitor view exit path (Back/Quit) without interrupting core flow on write errors.

## 3. Validation

- [ ] 3.1 Add tests for persistence round-trip (save then load) with multiple targets.
- [ ] 3.2 Add/adjust tests for graceful fallback when history file is missing or unreadable.
- [ ] 3.3 Run full test suite and confirm no behavior regression in TUI history isolation.
