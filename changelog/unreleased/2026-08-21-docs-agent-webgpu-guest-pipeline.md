### Docs — guest pipeline agent playbook (2026-08-21)

- Replace shape / semantic-L2 / midterm agent playbooks with [`docs/agent/webgpu-guest-pipeline.md`](../../docs/agent/webgpu-guest-pipeline.md) (P1 bind-group entries → P5 texture mip/sample/dimension)
- Remaining: `.\scripts\webgpu-guest-pipeline-remaining.ps1` or `python ./scripts/webgpu-guest-pipeline-remaining.py`; Cursor skill `webgpu-guest-pipeline`
- Closed queues stay closed: no re-hang, no re-cut labels/limits/sampler first-cut/canvas first-cut; still never file upstream GitHub issues
