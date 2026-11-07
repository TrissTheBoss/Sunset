## Summary

<!-- What changed and why? -->

## Evidence and scope

- [ ] This change does not claim unimplemented compatibility or gameplay.
- [ ] Compatibility behavior includes a reproducible reference scenario and exact target version, or is explicitly marked as unverified.
- [ ] No proprietary Minecraft assets or extracted files are included.
- [ ] Any new dependency has a documented use and license review.

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] `cargo test --workspace --locked`

<!-- List skipped checks and why. -->
