# jniLibs

Official layout: [`docs/mapping/artifacts.md`](../../docs/mapping/artifacts.md).

```powershell
.\scripts\build-native-android.ps1
.\scripts\verify-native-android.ps1 -RequireAll
```

Expected:

```text
jniLibs/
  arm64-v8a/libwasmtime_android_kt.so
  x86_64/libwasmtime_android_kt.so
  build-info.json
```

`.so` / `build-info.json` are gitignored; rebuild locally for instruments.
