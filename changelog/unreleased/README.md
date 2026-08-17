# Unreleased fragments

**English** | [中文](README.zh.md)

Feature / docs / fix PRs **add one markdown file in this directory**. **Do not** edit root [`CHANGELOG.md`](../../CHANGELOG.md).

Parallel short PRs all inserting at the top of `CHANGELOG.md` Unreleased will conflict; one new file per PR does not.

## File name

```text
YYYY-MM-DD-<slug>.md
```

- Date = expected merge day (or PR-open day).  
- `slug` = lowercase ASCII hyphenated, close to the branch name.  
- **Do not** edit someone else’s fragment.

## Body

Match root Changelog `###` sections. Do **not** wrap with `# Changelog` or `## Unreleased`:

```markdown
### Code — WASI 0.3 wasi:cli stdin read-via-stream smoke (2026-08-13)

- Register `wasi:cli/stdin@0.3.0#read-via-stream` …
- Fixture `fixtures/wasi/cli_stdin`; native `wasi_cli_stdin`
```

Title prefixes: `Code` / `Fix` / `Docs` / `Chore` / `BREAKING`.

## Rolling into the root Changelog

Maintainer (dedicated chore PR, not mixed with a feature):

```powershell
.\scripts\roll-changelog.ps1
```

The script inserts fragments (except `README.md` / `README.zh.md`) into `CHANGELOG.md` Unreleased by filename descending, then moves them to [`../archive/`](../archive/).
