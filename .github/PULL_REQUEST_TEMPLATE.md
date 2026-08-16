## Summary

<!-- 1–3 句：为什么改、解决什么问题。一 PR 一事。 -->

## Type

- [ ] docs
- [ ] feat（L1 / WASI / webgpu 切片）
- [ ] fix
- [ ] chore（CI / 工具链 / 模板）

## Checklist

- [ ] 已读 [`CONTRIBUTING.md`](../CONTRIBUTING.md) 与 [`docs/scheme/vcs-workflow.md`](../docs/scheme/vcs-workflow.md)
- [ ] 已新增 `changelog/unreleased/<yyyy-mm-dd>-<slug>.md`（**不要**改根 `CHANGELOG.md`）
- [ ] 未改枢纽文件（`CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md` / 本模板 / 根 README 索引），除非本 PR 就是改政策或工作流
- [ ] 未静默替换轨 A **默认 Demo runtime**；未引入 wasmtime4j 运行时依赖
- [ ] wasi:webgpu 功能切片：Guest 形状与钉版 WIT 同构（**不要**再交 host-fixed 过渡 u32）
- [ ] 触及 `native/`：本地或 CI `cargo test --locked --tests` 绿；新测试只加 `native/tests/*.rs`
- [ ] （若适用）只更新本切片主题文档（差距表 / tracking / 线程），不改「下一刀」总表

## Test plan

<!-- 列出跑过的命令，例如：
- cd native && cargo test --locked --tests
- ./gradlew :runtime-api:compileKotlin
-->

-

## Notes for reviewers

<!-- 风险、后续切片、不在本 PR 的范围 -->
