# Agent 手册：剩余 descriptor 语义

[English](webgpu-guest-semantics.md) | **中文**

**已关闭。** F1–F9 与本页点名 handle-0 / required-features 全列表已完成。当前队列：[`webgpu-guest-dawn.md`](webgpu-guest-dawn.md)（`.\scripts\webgpu-guest-dawn-remaining.ps1`）。不要重切挂名 / P1–P5 / F1–F9 / labels / limits / `create-sampler` first-cut / canvas first-cut。

史实：leftover optional 字段已进 Kotlin record；F8–F9 已让 Dawn 吃 pipeline constants 与 required-limits。仍开放的 Dawn 拷贝、write-mask / stencil / canvas 配置、`layout: auto`、WG-6 guest 绘制切片见当前手册。

空 compute `submit` 与 `vertex_index` 离屏三角不算本队列验收。史实车道表以[英文正文](webgpu-guest-semantics.md)为准。
