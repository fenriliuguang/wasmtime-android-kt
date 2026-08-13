# Unreleased 碎片

功能 / 文档 / 修复 PR **只在本目录新增一个 markdown 文件**，**不要**改根目录 [`CHANGELOG.md`](../../CHANGELOG.md)。

并行短刀同时往 `CHANGELOG.md` 的 Unreleased 顶部插行，几乎必冲突；每个 PR 一个新文件则互不重叠。

## 文件名

```text
YYYY-MM-DD-<slug>.md
```

- 日期用合入预期日（或开 PR 当日），四位年-两位月-两位日。  
- `slug` 用小写 ASCII、连字符，与分支名接近即可，例如 `wasi-cli-stdin`。  
- **不要**改别人的碎片；冲突了只处理自己的文件。

## 正文格式

与根 Changelog 的 `###` 节一致，**不要**再写 `# Changelog` 或 `## Unreleased`：

```markdown
### Code — WASI 0.3 wasi:cli stdin read-via-stream smoke (2026-08-13)

- Register `wasi:cli/stdin@0.3.0#read-via-stream` …
- Fixture `fixtures/wasi/cli_stdin`；native `wasi_cli_stdin`
```

标题前缀：`Code` / `Fix` / `Docs` / `Chore` / `BREAKING`。

## 滚入主干 Changelog

维护者（单线 chore，不要与功能 PR 抢同一 PR）：

```powershell
.\scripts\roll-changelog.ps1
```

脚本把本目录除 `README.md` 外的碎片按文件名倒序插入 `CHANGELOG.md` Unreleased，并移到 [`../archive/`](../archive/)。
