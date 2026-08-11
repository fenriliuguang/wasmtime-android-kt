# 贡献者构建与桌面开发壳

**中文** | （暂无 EN）

> M5：把「怎么本地搭环境 / 怎么用桌面 JVM 迭代 L1」从 Android 真机门禁里拆出来写清楚。  
> 正式 Android 复现仍以 [`build.md`](build.md) + [`mapping/artifacts.md`](mapping/artifacts.md) 为准。

## 1. 范围

| 做 | 不做 |
|----|------|
| 复现 Android `.so` + 仪器门禁 | 把桌面壳当成 CI / DoD 主门禁 |
| 可选：宿主 OS 编桌面 native，跑 JVM 冒烟 | Panama（[`scheme/non-goals.md`](scheme/non-goals.md) DG-1） |
| 改文档 / fixtures / Kotlin API 时同步 CHANGELOG | 静默替换轨 A 主验收（NG-1） |

原则：**Android-first**；桌面只是开发便利。

## 2. 前置（最小）

| 工具 | 钉死 / 说明 |
|------|-------------|
| JDK | 17+（Gradle Daemon 常用 21） |
| Rust | **1.97.1**（`native/rust-toolchain.toml`） |
| cargo-ndk | 仅 Android 交叉编译需要 |
| Android SDK + NDK `28.2.13676358` | 仅真机 / 模拟器仪器需要 |
| 轨 A mavenLocal | 测 M3+ L2 接线时需要（[`build-track-a-deps.md`](build-track-a-deps.md)） |

## 3. 推荐工作流

### 3.1 Android 主路径（门禁）

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

细节与 OEM 仪器注意见 [`build.md`](build.md)。

### 3.2 可选桌面开发壳（L1 迭代）

不装 NDK 时，可先在宿主 OS 编同一 cdylib，用桌面 JVM 验证 `loadLibrary` / 版本探针（及后续 JVM 单测）。

```powershell
# 1) 宿主 cargo → desktop/jniLibs/
.\scripts\build-native-host.ps1

# 2) 编译 Kotlin 面
.\gradlew.bat :runtime-api:compileKotlin :runtime-jni:compileKotlin

# 3) JVM 冒烟（脚本已把 java.library.path 指到 desktop/jniLibs）
.\gradlew.bat :runtime-jni:test
```

手工跑任意 JVM 进程时：

```powershell
# Windows 示例
java -Djava.library.path="$PWD\desktop\jniLibs" ...
```

| 项 | 值 |
|----|-----|
| 输出目录 | `desktop/jniLibs/`（**非正式**；gitignore） |
| Windows 文件 | `wasmtime_android_kt.dll` |
| Linux / macOS | `libwasmtime_android_kt.so` / `.dylib` |
| `System.loadLibrary` | 仍为 `wasmtime_android_kt` |

约束：

- **不**写入 `android/jniLibs/`，不替代双 ABI 正式布局。  
- Dawn / Surface / M4 上屏 **仍**走 Android 仪器。  
- 缺宿主库时 `:runtime-jni:test` 会失败并提示先跑 `build-native-host.ps1`。

## 4. 改代码时怎么对齐文档

| 变更 | 至少更新 |
|------|----------|
| 钉死工具链 / ABI | `docs/build.md`、`scheme/tech-stack.md`、CHANGELOG |
| 公开 API / 错误类型 | `scheme/api-stability.md`、`mapping/errors.md`、CHANGELOG |
| 与轨 A L2 接线 | `dual-track.md`、必要时差距文档 |
| WASI world 范围 | 先读 [`scheme/rfc-wasi-worlds.md`](scheme/rfc-wasi-worlds.md)；大扩面另开 RFC |

## 5. PR / 提交建议

1. 保持双轨隔离：本仓 PR **不**要求轨 A 为配合而破 sync-compat。  
2. 仪器绿优先于桌面绿；桌面壳失败但 Android 绿时，注明宿主环境即可。  
3. experimental `0.x`：破坏性变更写清 CHANGELOG（见 API 稳定性政策）。

## 6. 相关链接

- [`build.md`](build.md) — Android 构建复现  
- [`mapping/artifacts.md`](mapping/artifacts.md) — 正式 jniLibs  
- [`scheme/milestones.md`](scheme/milestones.md) — DoD  
- [`scheme/rfc-wasi-worlds.md`](scheme/rfc-wasi-worlds.md) — WASI world 路线图  
