# 差距：`wasi:webgpu` WIT ↔ NativeGpu ↔ Dawn C

[English](gap-webgpu-native-dawn.md) | **中文**

与英文冲突时以英文为准。`.so` 在时：boot + cube 热路径为 **Dawn**。其余 pin 方法（texture / compute / copy / map / query / bundle / error / 深度混合 / 索引绘制等）仍是 **Table**，收口见 BIND。Record 洞：compilation-hints、color-space、tone-mapping。
