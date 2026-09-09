### Chore — bump GAV to `0.1.3-SNAPSHOT` (2026-09-09)

- Coordinate **`0.1.3-SNAPSHOT`** after pressed `0.1.2`. Snapshot does not consume a Central release quota and may be overwritten. Same press pin: `--prebuilt` `libwebgpu_dawn.so` plus wasmtime opt-level 2. Consumers need the Central Portal snapshots repo (`maven("https://central.sonatype.com/repository/maven-snapshots/")`).
- Project skill [`.cursor/skills/bump-gav`](../../.cursor/skills/bump-gav/SKILL.md) for later coordinate cuts (current vs historical GAV, SNAPSHOT consume repo).
