# Agent 手册：Guest compute / 3D 管线编组

[English](webgpu-guest-pipeline.md) | **中文**

**已关闭。** P1–P5 与本页点名 first-cut 已完成。当前队列：[`webgpu-guest-dawn.md`](webgpu-guest-dawn.md)（`.\scripts\webgpu-guest-dawn-remaining.ps1`）。F1–F9 亦已关闭：[`webgpu-guest-semantics.md`](webgpu-guest-semantics.md)。不要重切挂名 / labels / limits / `create-sampler` first-cut / canvas first-cut。

史实：bind-group / BGL / vertex / depth / texture mip 已编组；sampler/view、pipeline-constant JNI、S1–S3 leftover、canvas first-cut、Dawn cite 亦已落地。仍开放的 Dawn 拷贝与 WG-6 guest 绘制切片见当前手册。

空 compute `submit` 与 `vertex_index` 离屏三角不算本队列验收。史实车道表以[英文正文](webgpu-guest-pipeline.md)为准。
