# 10k sandbox triage — see active worksheet

> **Active priority (2026-04-26):** strict-Perl dump parity. See
> [`SYNC_STATUS.md`](SYNC_STATUS.md) "Mission" and
> [`PERL_LOADFORMAT_AUDIT.md`](PERL_LOADFORMAT_AUDIT.md). Sandbox
> work continues opportunistically but is **not the gating front**;
> sandbox regressions during the dump-parity push are accepted per
> user directive.

The active sandbox triage worksheet is
[`sandbox_failures_SYNC_STATUS.md`](sandbox_failures_SYNC_STATUS.md).

That file tracks the focused 181-paper failure subset under
`~/data/sandbox_failures` (post-AR-flip, 2026-04-26 baseline) by
cluster, with a per-cluster fix-log. Workflow:

```
edit code → rebuild → ./tools/rerun_failures.sh → diff against
docs/sandbox_failure_181_triage.tsv → mark recovered papers [x]
```

This file ([`SANDBOX_TRIAGE.md`](SANDBOX_TRIAGE.md)) previously
held a session-by-session per-paper narrative through round 17.
Those narratives have been folded into commit messages and
`memory/project_session_history.md`. This file now exists only as
a redirect; do not write new triage notes here.
