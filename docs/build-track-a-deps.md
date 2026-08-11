# 轨 A L2 依赖（M3+）

轨 B 通过 **mavenLocal** 消费轨 A engineered artifacts，**不**依赖 `runtime-wasmtime` / wasmtime4j。

## 发布（轨 A 仓）

```powershell
cd ..\wasi-webgpu-jvm-mvp
.\gradlew.bat publishEngineeredToMavenLocal
```

坐标（`0.1.0-experimental`）：

- `io.github.fenriliuguang.wasi.webgpu.experimental:host-api`
- `io.github.fenriliuguang.wasi.webgpu.experimental:abi-cm`
- `io.github.fenriliuguang.wasi.webgpu.experimental:host-webgpu`（M4+ Dawn；Android AAR）

详见轨 A [`docs/maven-local.md`](../../wasi-webgpu-jvm-mvp/docs/maven-local.md)。

## 轨 B 解析

`settings.gradle.kts` 已含 `mavenLocal()`；版本钉在 `gradle/libs.versions.toml` → `wasiWebgpu`。
