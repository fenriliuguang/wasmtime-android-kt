# Store.postGfxVsync / closeGfxOnFrame are no-ops after close so a late Choreographer beat cannot crash the process.
# on-frame readiness polls (remaining=0) do not Drop the stream; a close during configure no longer traps the guest's first stream.read.
