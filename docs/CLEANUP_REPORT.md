# Repository cleanup report

This repository was consolidated from the physical-test v0.15.0 working tree into a GitHub-ready source snapshot.

## Removed

- Incremental overlay replacement directories from v0.13.0 through v0.15.0.
- Overlay apply scripts, overlay validators and overlay self-test runners.
- Patch-only fixture trees under `tests/`.
- Extracted package manifests: `MANIFEST.txt` and `README-APPLY.md`.
- Source-context scratch export: `rustmix-sleep-selector-context.txt`.
- `.DS_Store`, Python bytecode and `__pycache__` artifacts.
- Outdated milestone-by-milestone delivery documents that conflicted with the current state.

## Retained

- Active Rust source modules.
- Cargo metadata and ESP-IDF configuration.
- VS Code workspace settings.
- Maintained build, flash, host-test and SD-install helpers.
- Example removable-SD configuration files and sleep BMPs.
- Font-license text and current font notice.

## Added or repaired

- Current-state README.
- Consolidated CHANGELOG.
- Current architecture, board contract, SD setup, smoke test, known issues and GitHub upload documents.
- Missing `scripts/validate_source_contract.sh` referenced by build and validation helpers.
- `scripts/install-sd-examples.sh`.
- `scripts/package-release.sh`.
- GitHub Actions source-contract workflow.
- `.gitignore` updated so application `Cargo.lock` is committed while build outputs and local caches are excluded.
- Updated two stale host-render tests after cleanup: the enlarged typography sample now fits the mock display, and the Home chrome assertion now reflects the fixed dark footer.

## Source-code policy

No hardware-facing Rust module was removed during cleanup. The dead material identified in the uploaded archive was patch infrastructure, extracted overlay payloads, package fixtures and scratch artifacts. The physically verified firmware modules remain intact.
