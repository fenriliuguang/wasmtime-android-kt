# 差距：`wasi:webgpu` WIT ↔ NativeGpu ↔ Dawn C

[English](gap-webgpu-native-dawn.md) | **中文**

与英文冲突时以英文为准。`.so` 在时：boot + cube 热路径以及 BIND 覆盖的 pin 方法（texture / sampler / compute / copy / map / query / bundle / viewport / 索引绘制 / error scope 等）走 **Dawn**。缺 `.so` 仍是 **Table**。Record 洞：compilation-hints、color-space、tone-mapping。
