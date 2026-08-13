### Chore — disable Cursor/VS Code JDT autobuild (2026-08-13)

- Add workspace `.vscode/settings.json` so Red Hat Java does not import Gradle or copy `.kt` sources into `*/bin/`
- Ignore `**/bin/` so leftover JDT output stays untracked
