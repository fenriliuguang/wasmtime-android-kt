# 错误模型（轨 B L1）

**中文** | （暂无 EN）

> M5 切片：稳定 Kotlin 错误类型 + JNI 映射。  
> experimental CM host → L2 子集规则仍见 [`errors-m3.md`](errors-m3.md)。

## Kotlin 类型

| 类型 | `Kind` | 典型来源 |
|------|--------|----------|
| `WasmtimeApiException` | `API` | null/closed handle；注册时缺 callback |
| `WasmtimeCompileException` | `COMPILE` | `Component.compile` 失败 |
| `WasmtimeLinkException` | `LINK` | linker 定义 host / `instantiate` 失败 |
| `WasmtimeTrapException` | `TRAP` | export 调用 trap；含 host 回调失败 |
| `WasmtimeException`（基类） | 同上 | 可统一 `catch`；旧构造默认 `TRAP` |

均在 `runtime-api`：`io.github.fenriliuguang.wasmtime.android.api`。

## JNI 映射

Rust `native/src/error.rs` 按 kind 抛对应子类（`(Ljava/lang/String;)V`）。

| 操作 | 异常 |
|------|------|
| Engine/Store 创建失败、null handle、setter 参数非法 | `WasmtimeApiException` |
| `Component::new` | `WasmtimeCompileException` |
| `define_host` / `instantiate_async` | `WasmtimeLinkException` |
| `call*` / `run_concurrent` / 缺 export | `WasmtimeTrapException` |

## experimental host → L2（继承 M3）

| 来源 | 映射 |
|------|------|
| L2 `HostException`（回调内抛出） | host Err → guest **trap** → `WasmtimeTrapException` |
| L1 未注册 experimental host | trap（调用时） |
| L2 返回 `GpuHandle(0)` | 不预期；仪器断言非零 |

与轨 A 一致：**experimental** 轨不把 Host 失败抬成 guest-visible `result`；wasi `result` 编解码非本仓 M5 范围。

## 使用建议

```kotlin
try {
    instance.callUnitToU32(store, "run")
} catch (e: WasmtimeTrapException) {
    // guest / host-callback trap
} catch (e: WasmtimeException) {
    // compile / link / api
}
```

Kotlin 侧 `require`（closed handle）仍抛 `IllegalArgumentException`，**不**经 JNI。

## 明确不做（本切片）

- 把 trap 翻译成 guest `result<_,_>`
- Dawn / `HostException` 子类再细分到 L1 kind
- 稳定对外 1.0 错误码表（仍 experimental）
