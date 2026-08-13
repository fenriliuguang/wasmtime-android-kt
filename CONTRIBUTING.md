# 贡献指南

**中文** | 本仓为 experimental Android-first Wasm runtime（轨 B）。

感谢兴趣。正式协作约定以短命分支 + Pull Request 为准；**不要**直推 `main`（仓库 Ruleset 保护后将强制）。

## 必读

| 文档 | 内容 |
|------|------|
| [`docs/scheme/vcs-workflow.md`](docs/scheme/vcs-workflow.md) | 分支命名、PR 规则、并行矩阵、Ruleset / 开源就绪 |
| [`docs/contribute.md`](docs/contribute.md) | 本地构建、桌面开发壳、仪器门禁 |
| [`docs/scheme/long-term-plan.md`](docs/scheme/long-term-plan.md) | 现行长期计划（WASI 0.3 · wasi:webgpu · Wasmtime） |
| [`docs/scheme/non-goals.md`](docs/scheme/non-goals.md) | 非目标（勿静默替换轨 A 等） |

## 工作流（摘要）

1. Fork（外部）或从最新 `main` 拉短命分支：`docs/…` / `feat/…` / `fix/…` / `chore/…`。  
2. **一 PR 一事**；文档与行为变更同车；更新 `CHANGELOG.md` Unreleased。  
3. 推送分支并开 PR（目标 `main`）；通过 CI 后再合并。  
4. 合并策略：维护者默认 **squash merge**；合入后删除头部分支。  
5. **禁止**常驻多长期 `feature/*` 分叉后大爆炸合并。

## CI

Pull Request 与 `main` 推送会跑 GitHub Actions 工作流 **CI**（见 [`.github/workflows/ci.yml`](.github/workflows/ci.yml)）：

| Job | 内容 |
|-----|------|
| `native (cargo test)` | `native/` 下 `cargo test --locked --test m2_async_get --test p3_stream_read --test p3_stream_write --test wasi_random_u64 --test wasi_monotonic_now --test wasi_cli_stdout --test wasi_cli_stderr --test wasi_cli_stdin --test wasi_monotonic_wait_for --test wasi_monotonic_wait_until --test wasi_system_now --test wasi_webgpu_request_adapter --test wasi_webgpu_request_device`（CI 限并行以防 OOM） |
| `jvm (runtime-api compile)` | `:runtime-api:compileKotlin`（不依赖轨 A / Android SDK） |

本地建议：

```powershell
cd native
cargo test --all-targets

# 可选：纯 API 编译
.\gradlew.bat :runtime-api:compileKotlin
```

触及 `native/` 的变更：合入前至少保证上述 cargo 测试绿。Android 仪器 / 双 ABI 构建仍按 [`docs/contribute.md`](docs/contribute.md) 与 [`docs/build.md`](docs/build.md) 在设备或本机复现。

## 权限与协作模型

| 角色 | 建议权限 | 如何贡献 |
|------|----------|----------|
| 维护者 | Admin / Maintain | 短分支 + PR；合并经 Ruleset |
| 受信协作者 | **Write**（不要给 Admin，除非必要） | 同仓短分支 + PR；**不要**直推 `main` |
| 外部贡献者 | 无写权限 | **Fork** → 短分支 → PR 回本仓 |

说明：

- 外人**不必**、也**不应**被授予直接推 `main` 的权限。  
- Write 仍可能在无 Ruleset 时直推分支；保护靠 **Ruleset 强制 PR**（见 `vcs-workflow.md`）。  
- 轨 A（`wasi-webgpu-jvm-mvp`）另仓；本仓 PR **不得**要求轨 A 破坏 sync-compat 锁死条款。

## 行为准则（简）

- experimental：不宣称合规 wasi:webgpu / 生产级 runtime（未另开 RFC 前）。  
- 真 CM async / WASI 0.3 异步禁止用 sync-compat 冒充。  
- 友善、具体的 review；安全与许可证问题请私下联系维护者。

## 许可

贡献默认与本仓相同许可证：**Apache License 2.0**（[`LICENSE`](LICENSE)、[`NOTICE`](NOTICE)）。  
第三方依赖见 [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)。提交即表示你有权按该许可贡献。
