# 贡献者构建与桌面开发壳

[English](contribute.md) | **中文**

本地环境与桌面 JVM 迭代 L1。正式 Android 复现以 [`build.md`](build.md) 为准。英文为正文。

Android 主路径：

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

GPU 仪器仍依赖 **未发布** Host 构件，见 [`blocked-gpu-host.md`](blocked-gpu-host.md)。**不要在文档 PR 里删除这些 Gradle 依赖。**

禁止引入 wasmtime4j 作为运行时。用户可见变更只写 `changelog/unreleased/` 碎片。
