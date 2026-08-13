### Chore — 并行短刀：冻结文档/CI 枢纽 (2026-08-13)

- CI `native` job 改为 `cargo test --locked --tests`：新 `native/tests/*.rs` 自动入闸，功能 PR 不必改 `.github/workflows/ci.yml`
- 用户可见变更改为新增 `changelog/unreleased/<date>-<slug>.md`；禁止在功能 PR 里改根 `CHANGELOG.md`（维护者 `.\scripts\roll-changelog.ps1` 滚入）
- CONTRIBUTING / PR 模板 / README / vcs-workflow 去掉会随每刀改写的测试枚举与 Unreleased 追加
