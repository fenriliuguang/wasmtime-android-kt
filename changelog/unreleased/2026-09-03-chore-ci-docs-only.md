### Chore — skip heavy CI on markdown/changelog-only PRs (2026-09-03)

- Detect job: change set of only `*.md` and `changelog/**` skips cargo test / NDK / AAR; aggregate check **`CI`** still succeeds (Ruleset name unchanged). Any other path runs the four jobs. Fail-open to heavy when the file list is empty or the base SHA is missing.
