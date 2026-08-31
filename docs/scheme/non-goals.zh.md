# 非目标

[English](non-goals.md) | **中文**

硬边界：不以 wasmtime4j 为运行时；不重造完整 Kotlin WebGPU 客户端；不以全量 WASI/CTS 为 KPI；不宣称合规产品（L5 是 **0.x 产品子集**）；不做 `0.0.x-preview` Central（P010-PUB 后坐标 `0.1.0` + `publish.yml`；secrets 缺失时不要强发）；**不重写第二套 Dawn**（包装/适配**一份** Dawn 允许：androidx JNI 剩路与/或 native C，见 native-dawn 手册）；不用 Latch 冒充实 async；不把 wasi-gfx **升为 P0**（最小帧循环是 **0.1.0 门禁**，见 gfx RFC）；新切片禁止 host-fixed u32。

已从表中删除的双产品条款见归档。GPU 后端代码依赖见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。
