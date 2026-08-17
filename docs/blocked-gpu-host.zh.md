# 阻断：未发布的 GPU Host 库

[English](blocked-gpu-host.md) | **中文**

本页是停车标志，不是构建指南。仪器测试与 `runtime-jni` 仍依赖 **不属于本仓** 的 mavenLocal 构件：

| Gradle 别名 | 坐标 |
|-------------|------|
| `libs.wasi.webgpu.host.api` | `io.github.fenriliuguang.wasi.webgpu.experimental:host-api:0.1.0-experimental` |
| `libs.wasi.webgpu.host.webgpu` | `…:host-webgpu:0.1.0-experimental`（APK 内 Dawn `.so`） |
| `libs.wasi.webgpu.abi.cm` | `…:abi-cm:0.1.0-experimental` |

**文档 PR 不得删除这些依赖。** 目标形态见 [`scheme/rfc-pluggable-gpu-backend.md`](scheme/rfc-pluggable-gpu-backend.md)。Dawn 字节如何进入 `:host-dawn` 仍须拍板：vendor / 发布 / 仅 mavenLocal。
