# Acceptance Test Run

Date: February 15, 2026

Command:

```bash
cargo test acceptance_ -- --nocapture
```

Result:

- `acceptance_flatten_and_misc_routing`: passed
- `acceptance_collision_rename_policy`: passed
- `acceptance_cleanup_protects_category_and_root`: passed
- `acceptance_dry_run_matches_run_destinations`: passed
- `acceptance_undo_last_run_best_effort`: passed

Summary: 5 passed, 0 failed.

These tests run against real temporary filesystem folders and validate the core product contract:

- configurable `sortRoot` behavior
- flatten-mode file moves
- extension classification (including unknown/no-extension handling)
- rename collision policy
- protected folder cleanup guarantees
- dry-run/run parity
- undo-last-run best effort behavior
