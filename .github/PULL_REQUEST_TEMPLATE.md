## Summary

<!-- 1–3 句：为什么改、解决什么问题。一 PR 一事。 -->

## Type

- [ ] docs
- [ ] feat（L1 / WASI / webgpu 切片）
- [ ] fix
- [ ] chore（CI / 工具链 / 模板）

## Checklist

- [ ] 已读 [`CONTRIBUTING.md`](../CONTRIBUTING.md) 与 [`docs/scheme/vcs-workflow.md`](../docs/scheme/vcs-workflow.md)
- [ ] CHANGELOG Unreleased 已更新（用户可见行为 / 文档政策）
- [ ] 未静默替换轨 A 主验收；未引入 wasmtime4j 运行时依赖
- [ ] 触及 `native/`：本地或 CI `cargo test` 绿
- [ ] （若适用）更新差距表 / tracking / 线程文档

## Test plan

<!-- 列出跑过的命令，例如：
- cd native && cargo test --test p3_stream_read --test p3_stream_write --test wasi_random_u64 --test wasi_monotonic_now --test wasi_cli_stdout
- ./gradlew :runtime-api:compileKotlin
-->

-

## Notes for reviewers

<!-- 风险、后续切片、不在本 PR 的范围 -->
