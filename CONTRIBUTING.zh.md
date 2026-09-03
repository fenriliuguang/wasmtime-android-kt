# 贡献指南

[English](CONTRIBUTING.md) | **中文**

短命分支 + PR；不要直推 `main`。英文为正文。`release/0.1.0` 可作为维护分支，且从不在该分支发包。

必读：[`docs/scheme/rfc.md`](docs/scheme/rfc.md)、[`docs/scheme/guest-shape.md`](docs/scheme/guest-shape.md)、[`docs/scheme/claim-010.md`](docs/scheme/claim-010.md)、[`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md)。

用户可见变更只新增 `changelog/unreleased/<date>-<slug>.md`。禁止改根 `CHANGELOG.md`。新测试只加 `native/tests/*.rs`。

CI（`main` / `release/0.1.0`）：host `cargo test`、JVM compile、NDK `.so`、AAR assemble。仅改 `*.md` / `changelog/` 时这四项 skipped，聚合检查 `CI` 仍绿。真机 instruments 与仓外 example 在本机跑，作为 GitHub Environment `release` 的发包门禁。`0.1.0` 已发布。后续发包仅 `main` 上的 `v*` 标签或从 `main` 手动触发。

与英文冲突时以英文为准。
