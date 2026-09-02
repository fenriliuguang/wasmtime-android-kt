# 节拍同步

[English](gfx-hitch-checklist.md) | **中文**

当前路径上，Host `postGfxVsync` → NativeGpu present 与显示节拍 **1:1**。
**本仓节拍同步现阶段没有剩余问题。**

keep-3 回收、Fifo、H8 二次 present 空操作仍是产品不变量。不要 vendor 仓外
demo。不要给上游提 GitHub issue。
