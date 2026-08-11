# 如何构建（轨 B / M0+）

**中文** | 对齐轨 A NDK / AGP / Rust 钉死值。修订须同步 [`scheme/tech-stack.md`](scheme/tech-stack.md) + CHANGELOG。

## 前置

| 工具 | 版本（钉死） |
|------|----------------|
| JDK | 17+（Gradle Daemon 经 Foojay 用 **21**） |
| Android SDK | 含 `platforms;android-36` |
| NDK | **28.2.13676358** |
| Rust | **1.97.1**（`native/rust-toolchain.toml`） |
| cargo-ndk | `cargo install cargo-ndk` |
| Gradle | Wrapper **9.6.1**（仓库自带） |

环境变量（任选其一指向 SDK）：`ANDROID_SDK_ROOT` / `ANDROID_HOME`。  
或在仓库根创建 gitignored 的 `local.properties`：

```properties
sdk.dir=C\:\\Users\\<you>\\AppData\\Local\\Android\\Sdk
```

```powershell
sdkmanager --install "ndk;28.2.13676358"
rustup toolchain install 1.97.1
cargo install cargo-ndk
```

## 1. 构建 Android `.so`

正式布局见 [`mapping/artifacts.md`](mapping/artifacts.md)。

```powershell
cd d:\projects\wasmtime-android-kt
.\scripts\build-native-android.ps1
```

默认产出（双 ABI + 元数据）：

```text
android/jniLibs/arm64-v8a/libwasmtime_android_kt.so
android/jniLibs/x86_64/libwasmtime_android_kt.so
android/jniLibs/build-info.json
```

仅 arm64：

```powershell
.\scripts\build-native-android.ps1 -Targets arm64-v8a
```

校验（不编译；`-RequireAll` 要求双 ABI）：

```powershell
.\scripts\verify-native-android.ps1 -RequireAll
```

说明：

- `JNI_OnLoad` 返回 **`JNI_VERSION_1_6`**（ART 拒 1_8）。
- Bionic 无 `libpthread`：脚本用 `native/link-stubs/libpthread.so` → `INPUT(-lc)`。
- Windows 交叉编译默认 `CARGO_PROFILE_RELEASE_OPT_LEVEL=0`（规避 rustc ACCESS_VIOLATION），再用 `llvm-strip`。
- 构建结束写入 `build-info.json` 并对本次 `-Targets` 跑校验。

## 2. 编译 JVM / Android 模块

需先完成步骤 1（否则 `smoke-app` 仪器缺 `.so`）。

```powershell
.\gradlew.bat :runtime-api:compileKotlin :runtime-jni:compileKotlin :android:assembleDebug :smoke-app:assembleDebug
```

## 3. M0 仪器（真机 / 模拟器）

设备 ABI 需匹配已产出的 `.so`（真机优先 **arm64-v8a**）。

```powershell
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

用例：`LoadLibraryInstrumentedTest` — `loadLibrary` + `nativeWasmtimeVersion` 非空。

OEM / UTP 竞态时可改用 `adb shell am instrument`（经验同轨 A）。

## 模块

| 模块 | 角色 |
|------|------|
| `runtime-api` | 公共常量 / 将来 Engine API（无 Android 依赖） |
| `runtime-jni` | `NativeLoader` / JNI 声明 |
| `android` | AAR + `jniLibs` |
| `smoke-app` | 最小 Activity + 仪器 |
| `native/` | Rust cdylib（非 Gradle 子项目） |

## 4. 可选：桌面开发壳

无 NDK 时可用宿主 cdylib + JVM 冒烟迭代 L1（**非正式门禁**）：

```powershell
.\scripts\build-native-host.ps1
.\gradlew.bat :runtime-jni:test
```

完整贡献者流程见 [`contribute.md`](contribute.md)。

## 明确不做

- **不**依赖 `wasmtime4j` / 轨 A `runtime-wasmtime`
- 桌面壳 **不**替代 Android 仪器门禁（见 [`scheme/milestones.md`](scheme/milestones.md)）
