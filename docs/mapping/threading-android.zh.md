# Android 线程契约

[English](threading-android.md) | **中文**

真 CM async：Guest 可挂起，Host 稍后 complete future。Dawn / `ANativeWindow` 仍须在 **单一 GpuThread** 上使用。禁止在 ART 主线程做重 compile/instantiate。

今日 GPU 对象来自仓内 `:host-dawn` + `androidx.webgpu`（[`../blocked-gpu-host.md`](../blocked-gpu-host.md)）；无论后端是谁，上述线程规则仍适用。

细节与 M4 钉死以英文正文为准。

帧循环仅建议（非计划 / 非 P2 刀）：[`frame-loop-suggestion.zh.md`](frame-loop-suggestion.zh.md)。
