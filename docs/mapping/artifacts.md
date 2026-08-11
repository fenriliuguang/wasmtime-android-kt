# Native 产物布局（轨 B）

**中文** | （暂无 EN）

> M5：钉死 `libwasmtime_android_kt.so` 的正式目录、ABI 与校验入口。

## 正式布局

根目录：`android/jniLibs/`（由 `:android` AAR / `smoke-app` 消费；**gitignored**）。

```text
android/jniLibs/
  arm64-v8a/libwasmtime_android_kt.so   # 真机主 ABI（必选）
  x86_64/libwasmtime_android_kt.so      # 模拟器 / 桌面 Android（正式双 ABI 必选）
  build-info.json                       # 构建元数据（脚本生成，gitignore）
```

| 项 | 值 |
|----|-----|
| `System.loadLibrary` | `wasmtime_android_kt`（见 `NativeLibraryNames.WASMTIME_ANDROID_KT`） |
| 文件名 | `libwasmtime_android_kt.so` |
| 最小 API | 24（与 `cargo ndk --platform` 一致） |
| NDK | `28.2.13676358` |
| Rust | `1.97.1` |

## 构建

```powershell
# 正式双 ABI（默认）
.\scripts\build-native-android.ps1

# 仅真机迭代
.\scripts\build-native-android.ps1 -Targets arm64-v8a

# 校验已有产物（不编译）
.\scripts\verify-native-android.ps1
.\scripts\verify-native-android.ps1 -RequireAll   # 要求 arm64 + x86_64 均存在
```

构建脚本在成功后会：

1. `llvm-strip --strip-unneeded`（可用 `-SkipStrip` 跳过）  
2. 写入 `android/jniLibs/build-info.json`  
3. 对本次 `-Targets` 跑校验  

## `build-info.json` 字段

| 字段 | 说明 |
|------|------|
| `libraryFile` | `libwasmtime_android_kt.so` |
| `loadLibrary` | `wasmtime_android_kt` |
| `ndkVersion` / `apiLevel` / `rustToolchain` | 钉死工具链 |
| `abis.<abi>.bytes` / `sha256` | 各 ABI 大小与内容哈希 |
| `builtAt` | UTC ISO-8601 |

## 校验规则（`verify-native-android.ps1`）

- 路径：`android/jniLibs/<abi>/libwasmtime_android_kt.so`  
- 文件存在且 `bytes >= 1 MiB`（防止空/截断产物）  
- `-RequireAll`：必须同时有 `arm64-v8a` 与 `x86_64`  
- 若存在 `build-info.json`，可选核对 sha256（脚本默认核对所列 ABI）  

## 可选：桌面宿主产物（非正式）

开发便利见 [`../contribute.md`](../contribute.md)：

```powershell
.\scripts\build-native-host.ps1
# → desktop/jniLibs/wasmtime_android_kt.dll|.so|.dylib + build-info.json
```

- **不**进入 AAR / 仪器门禁；**不**替代上表双 ABI。  
- JVM：`-Djava.library.path=<repo>/desktop/jniLibs`（`:runtime-jni:test` 已配置）。

## 非目标

- 不上传 Maven / 不宣布对外二进制发布  
- 不在本仓提交 `.so`  
- 不增加 `armeabi-v7a` / `x86`（除非单独 RFC）  
