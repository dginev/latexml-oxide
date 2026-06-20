# `cortex_worker` harness mode — fleet orchestration & memory caps

How latexml-oxide's `cortex_worker` binary orchestrates a robust, self-contained
fleet of conversion workers, and the layered memory guards that bound each one.

* **Mechanism** (the reusable supervision/respawn/`RLIMIT_AS` library):
  pericortex, `docs/HARNESS.md`.
* **Operator deployment** (bring the fleet up against a running dispatcher):
  CorTeX `MANUAL.md` §7.
* **In-process runaway guards** (the byte budget / cycle guards *inside* one
  conversion): `docs/archive/MEMORY_GUARD_HARDENING_2026-06-09.md`.

## The two modes of `cortex_worker`

`cortex_worker` implements `pericortex::worker::Worker` and can run as either:

1. **A single worker** (default): connects to the dispatcher and converts.
   `--pool-size 1` is one conversion per process; `--pool-size N` runs N
   conversion threads in **one** process sharing **one** RAM ceiling — avoid it
   for production (the shared ceiling false-positives the memory guards on every
   in-flight paper; see pericortex `docs/HARNESS.md`).
2. **A self-supervising harness** (`--harness`): becomes a process supervisor.
   It does no conversion itself; it spawns and keeps alive a fleet of
   single-conversion (`--pool-size 1`) **child processes** — copies of its own
   `current_exe()` with the dispatcher/profile flags forwarded and `--harness`
   **omitted** (so there is no fork bomb) — respawning any that die, until
   SIGTERM/SIGINT tears the fleet down cleanly.

`--harness` does not reimplement supervision: it calls
`pericortex::harness::supervise()`. The harness *logic* lives in pericortex
(reusable across CorTeX workers); `cortex_worker` is the worker that *drives*
it. "Self-supervising" and "the pericortex harness" are the same design viewed
from two layers — not competing options.

## Harness CLI

| Flag | Default | Meaning |
| --- | --- | --- |
| `--harness` | off | Run as the supervising harness instead of converting. |
| `--workers N` | CPU-derived¹ | Number of single-conversion child processes to keep alive. A **deliberate over-commit** (see below) — explicit override. |
| `--child-mem-limit-mb MB` | `5632` | Per-child **address-space** ceiling, enforced via `setrlimit(RLIMIT_AS)` before each child execs (`0` disables). Contains a *single* runaway job. (Was `8192` until a 72-worker sweep let the aggregate hit 207 GB → kernel OOM; now a ~4 GB-RSS cap. The binary's `--help` is the authoritative default.) |
| `--mem-pressure-floor-mb MB` | auto³ | Fleet **memory-pressure governor** floor. Contains the *aggregate* (a heavy cluster). Omit = auto; `0` = disable. |
| `--max-rss-mb MB` | auto² | Per-child polled **RSS** soft guard (forwarded to children). |

¹ `pericortex::harness::default_worker_count()`: CPU count minus a reservation
for the OS + dispatcher (−4 above 16 cores, −2 above 4, −1 above 2). This is a
**deliberate over-commit** — most jobs use a fraction of the per-child cap, so
sizing the fleet to the cap would idle most of the box; the governor (below)
makes the over-commit safe against the rare heavy cluster.
² In harness mode the child's `--max-rss-mb` is auto-set to
`--child-mem-limit-mb − 256` so the soft guard fires just before the hard cap.
With the 5632 default the soft guard sits at 5376 MiB RSS, and mimalloc reserves
~1–1.5 GiB of VSZ above RSS, so the cap trips at a ~4 GB *resident* ceiling — a
legitimate ~4 GB paper completes and a runaway is killed early (the deliberate
trade behind lowering it from 8192, which once OOMed the box at 207 GB aggregate).
³ Auto = `max(one per-child cap, 10% of physical RAM)`: shed when free RAM
drops below that, resume past 1.5×. Set explicitly to tune; `0` disables the
governor (then rely on the per-child cap + an external cgroup).

Example (one command brings up the whole fleet):

```bash
cortex_worker --harness \
  --service oxidized-tex-to-html \
  --source-address tcp://dispatcher:51695 \
  --sink-address   tcp://dispatcher:51696 \
  --profile ar5iv
# --workers defaults to the CPU-derived over-commit; --mem-pressure-floor-mb
# defaults to max(per-child cap, 10% of RAM). Add either flag only to override.
# Production: prefer CorTeX's canonical launcher (scripts/run_worker.sh +
# cortex-worker.service) — it pins --workers and leaves these guards at the validated defaults.
```

The dispatcher/profile flags (`--source-address`, `--sink-address`, `--service`,
`--profile`, `--timeout`, `--message-size`, `--preload`, `--path`, `--no-pmml`,
`--no-mathtex`, `--limit`, verbosity) are forwarded to every child. Forwarding
`--limit N` recycles each child after N tasks (the harness respawns it),
bounding any slow per-process memory creep.

## Layered memory guards (defense in depth)

Guards 1–4 bound a **single conversion** (per-process); guard 5 bounds the
**fleet aggregate**. Innermost to outermost:

1. **Gullet/stomach cycle guards + stomach byte budget** (`STOMACH_BOX_BYTES_BUDGET`,
   ~3.2 GB of estimate) — detect a runaway *during* digestion and `Fatal` with a
   structured `Stomach:MemoryBudget`/`Stomach:Recursion`. Portable, fires before
   any RSS ceiling. See `docs/archive/MEMORY_GUARD_HARDENING_2026-06-09.md`.
2. **Polled RSS soft guard** (`--max-rss-mb`, the shared `Watchdog`) — samples
   `/proc/self/status` `VmRSS` and exits `137` with a `Fatal:oom:rss` log line
   and a `Status:conversion:3` artifact. Linux-only (reads `/proc`).
3. **Alloc-error hook** (`custom_alloc_error_hook`) — when *any* allocation
   returns null (e.g. an `ENOMEM` from the hard cap below), emits
   `Fatal:oom:alloc_failed` and exits `137`. Portable.
4. **Hard `RLIMIT_AS` cap** (`--child-mem-limit-mb`, applied by the harness) —
   the kernel refuses allocations past the address-space ceiling, which surfaces
   as `ENOMEM` → guard #3 → clean attributable exit → respawn. Contains a
   **single** runaway job.
5. **Fleet memory-pressure governor** (`--mem-pressure-floor-mb`, in the harness
   process) — samples system `MemAvailable`; below the floor it SIGTERMs the
   **largest-RSS** child (its task re-leased) and pauses respawns until memory
   recovers past 1.5× the floor. Contains the **aggregate** — a cluster of
   concurrently-heavy jobs that each stay under guard #4 but together threaten
   the host. This is what makes the deliberate worker over-commit safe.

Guards 1–4 are deliberately staggered so the **inner, attributable** ones fire
first: a paper is reported as a logged `Status:conversion:3` (and its task
re-leased) rather than vanishing in a silent kill. Guard 5 likewise *chooses* a
victim and re-leases it, in place of an indiscriminate kernel OOM-kill.

For a hard host-level backstop beneath all five, run the harness inside a cgroup
`memory.max` (a memory-limited container, or `systemd-run --scope -p
MemoryMax=…`); they compose — the cgroup caps the host, the governor sheds
proactively before it, and `RLIMIT_AS` caps each child.

### `RLIMIT_AS` caps address space, not RSS

The worker uses **mimalloc** (`#[global_allocator]`) to avoid glibc arena-mutex
contention under the multi-process fleet. mimalloc reserves virtual address
space above its resident set, so a `--child-mem-limit-mb 4096` cap (which bounds
**VSZ**) trips at a *true RSS* somewhat **below** 4 GiB. This is the right
trade: it errs toward killing a runaway early, and it is the only privilege-free,
portable `setrlimit` knob (`RLIMIT_RSS` is a Linux no-op). For a *precise* 4 GiB
*RAM* cap, run the harness inside a cgroup `memory.max` (a memory-limited
container, or `systemd-run --scope -p MemoryMax=4G`); the cgroup caps the
aggregate while each child's `RLIMIT_AS` caps the individual — they compose.
Full mechanism + rationale: pericortex `docs/HARNESS.md`.

## Production recommendation

* **Default: `cortex_worker --harness`.** Self-contained, single binary, single
  command, portable to any host or bare container, shipping per-child
  `RLIMIT_AS` + the fleet memory-pressure governor + crash-loop backoff +
  SIGCHLD-prompt respawn + `PR_SET_PDEATHSIG` (no orphans) + graceful shutdown,
  with no external dependency — consistent with the project's
  self-contained-binary design principle.
* **Under systemd/k8s:** either run `--harness` inside a memory-limited
  container/pod (per-child `RLIMIT_AS` + an aggregate cgroup cap — defense in
  depth), or let the platform supervise `--pool-size 1` workers directly and use
  cgroup `MemoryMax`/`resources.limits.memory` for a true-RSS cap. Use whichever
  the platform makes ergonomic; `--harness` is the portable fallback.
* **Never `--pool-size N` for production** — it shares one RAM ceiling across N
  conversions.

## Exit-code reference (per child)

| Code | Meaning | Source |
| --- | --- | --- |
| `0` | success / warnings | normal |
| `124` | wall-clock timeout | `Watchdog` |
| `137` | memory ceiling / alloc failure | `Watchdog` RSS guard or `custom_alloc_error_hook` (incl. `RLIMIT_AS` `ENOMEM`) |
| `70` | 5 consecutive caught panics → clean restart | `cortex_worker` `MAX_CONSECUTIVE_PANICS` |

All map to a `Status:conversion:3` failure the dispatcher records; the lease
reaper recovers the in-flight task and the harness respawns the worker.
