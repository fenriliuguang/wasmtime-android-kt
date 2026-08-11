# M3 错误映射策略（subset）

**中文** · 对齐轨 A [`errors-async.md`](../../../wasi-webgpu-jvm-mvp/docs/mapping/errors-async.md) 的可执行子集。

> **全文已并入 [`errors.md`](errors.md)。** 本页保留 M3 主链范围说明。

## 范围

M3 仅覆盖 experimental CM 主链一条：

`experimental:webgpu-cm/host@0.8.0#request-adapter` → L2 `WasiWebGpuHost.requestAdapter()` → `u32` rep。

## 规则（当前）

| 来源 | 映射 | 说明 |
|------|------|------|
| L2 `HostException`（任意子类） | **CM trap** | JNI 抛 `WasmtimeTrapException` |
| L2 返回 `GpuHandle(0)` | **不预期**；Cpu 路径返回非零 | 仪器断言 `rep != 0` |
| L1 未注册 callback | trap | |
| Component 编译 / 实例化失败 | `WasmtimeCompileException` / `WasmtimeLinkException` | |

## 明确不做（M3）

- wasi:webgpu `result` / `option` 编解码（`HostErrorMapping` 全量）
- 把 trap 翻译成 guest-visible `result<_,_>`  
- Dawn 特有错误码细分
