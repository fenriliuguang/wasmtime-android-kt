## Summary

<!-- 1–3 sentences: why this change, what it solves. One PR, one thing. -->

## Type

- [ ] docs
- [ ] feat (L1 / WASI / webgpu slice)
- [ ] fix
- [ ] chore (CI / toolchain / templates)

## Checklist

- [ ] Read [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- [ ] Added `changelog/unreleased/<yyyy-mm-dd>-<slug>.md` (**do not** edit root `CHANGELOG.md`)
- [ ] Did not churn hub files (`CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md` / this template / root README index) unless this PR **is** policy or workflow
- [ ] Did not introduce wasmtime4j as the runtime
- [ ] Did **not** delete unpublished GPU host Gradle deps without an explicit decision ([`docs/blocked-gpu-host.md`](../docs/blocked-gpu-host.md))
- [ ] wasi:webgpu feature slices: guest shape isomorphic with pinned WIT (**no** new host-fixed transitional `u32`)
- [ ] Touched `native/`: local or CI `cargo test --locked --tests` green; new tests only as `native/tests/*.rs`
- [ ] (If applicable) updated only this slice’s topic docs (gap / threading), not a “next cut” master list

## Test plan

<!-- Commands you ran, e.g.:
- cd native && cargo test --locked --tests
- ./gradlew :runtime-api:compileKotlin
-->

-

## Notes for reviewers

<!-- Risks, follow-up slices, out of scope -->
