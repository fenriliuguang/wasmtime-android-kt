# 如何构建

[English](build.md) | **中文**

钉死值须与 [`scheme/tech-stack.md`](scheme/tech-stack.md) 同步。英文为正文。

| 工具 | 版本 |
|------|------|
| NDK | **28.2.13676358** |
| Rust | **1.97.1** |
| Gradle Wrapper | **9.6.1** |

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

GPU 仪器走仓内 `:host-dawn` + `androidx.webgpu`：[`blocked-gpu-host.md`](blocked-gpu-host.md)。不依赖 wasmtime4j 运行时。
