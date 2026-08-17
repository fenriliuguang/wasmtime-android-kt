# M4 相对轨 A cube 差距清单

**中文** | （暂无 EN）

> 轨 B M4 专用 render smoke（clear→present）vs 轨 A `cube_cm.wasm` / `WasmtimeCmCubeInstrumentedTest`。  
> **不**宣称取代轨 A 主验收。

## 已对齐（本切片）

| 项 | 状态 |
|----|------|
| Android Surface + GpuThread | ✅ `DawnRenderSmokeInstrumentedTest` |
| Dawn L2（`host-webgpu` / `DawnWasiWebGpuHost`） | ✅ |
| `windowFromSurface` → native window | ✅ |
| clear + submit + present + unconfigure | ✅ 专用 Guest |
| 线程契约（UI 投递 / GpuThread 碰 Dawn） | ✅ 见 [`threading-android.md`](threading-android.md) |

## 相对轨 A cube 仍缺

| 缺口 | 说明 |
|------|------|
| WIT **resource** 类型 | M4 仍用扁平 `u32` rep；cube 用 `adapter`/`device`/… + `[method]…` |
| 完整 `experimental:webgpu-cm/host@0.8.0` | 仅 clear→present 子集；无 pipeline / buffer / bind-group / depth |
| Guest `run-cube` / 帧循环 | 无纹理立方体、MVP、`init-cube`/`draw-frame`/`drop-cube` |
| `result<_, string>` 导出 | smoke 用 `u32` 状态码（0=ok） |
| Descriptor 编组 | 无 `CmDescriptorParsers` 级 record/list/string |
| 真 CM async + Dawn | 仍 sync-compat（回调内等待）；M2 async 未接 GPU |
| 像素 / 帧内容断言 | 仅 present 成功；无读回或截图门禁 |

## 后续切片（建议）

1. WIT resource 表 + 方法名对齐 `AbiCm.Func`  
2. 接 `cube_cm.wasm` one-shot（仍独立仪器类名）  
3. 可选短帧循环；仍不覆盖轨 A 主脚本职责  
