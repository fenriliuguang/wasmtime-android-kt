# jniLibs

Place `libwasmtime_android_kt.so` here after running:

```powershell
.\scripts\build-native-android.ps1
```

Expected layout:

```text
jniLibs/
  arm64-v8a/libwasmtime_android_kt.so
  x86_64/libwasmtime_android_kt.so   # optional / emulator
```

`.so` files are gitignored; rebuild locally for instruments.
