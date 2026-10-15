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
  --service oxidized_tex_to_html \
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
3. **Alloc-error hook** (`custom_alloc_error_hook`) — when any *Rust* allocation
   returns null (e.g. an `ENOMEM` from the hard cap below; C libraries such as libxml2 call `malloc` directly and never
   reach it), emits
   `Fatal:oom:alloc_failed` and exits `137`. Portable. The memory is spent, so the hook allocates nothing and takes no
   lock: its line is formatted on the stack and written straight to fd 2 (`watchdog::write_stderr_unlocked`), it prints
   no backtrace, and it ends with `_exit` (`watchdog::exit_now`). A re-entry (its own sweep failing to allocate) ends
   the process at once (`watchdog::enter_alloc_failure_hook`). Before 67m it formatted a backtrace under the fleet's
   `RUST_BACKTRACE=1` and could re-enter itself and wait forever on the backtrace lock, with stderr held, so the
   watchdog's timeout blocked too; the harness's 360 s unresponsive kill then re-leased the paper (SYNC_STATUS 67m).
4. **Hard `RLIMIT_AS` cap** (`--child-mem-limit-mb`, applied by the harness) —
   the kernel refuses allocations past the address-space ceiling, which surfaces
   as `ENOMEM` → guard #3 → clean attributable exit → respawn. Contains a
   **single** runaway job.
5. **Fleet memory-pressure governor** (`--mem-pressure-floor-mb`, in the harness
   process) — samples system `MemAvailable`; below the floor it SIGTERMs the
   **largest-RSS** child (its task reported `Fatal:oom:shed`) and pauses respawns until memory
   recovers past 1.5× the floor. Contains the **aggregate** — a cluster of
   concurrently-heavy jobs that each stay under guard #4 but together threaten
   the host. This is what makes the deliberate worker over-commit safe.

Guards 1–4 are deliberately staggered so the **inner, attributable** ones fire
first: a paper is reported as a logged `Status:conversion:3` rather than vanishing in a silent
kill. Guard 5 likewise *chooses* a victim, in place of an indiscriminate kernel OOM-kill. A
child that guards 2–5 end returns its result through the harness (see "Exit reports" below).

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

The reservations also **accumulate** in a long-lived child. In run 336 (2026-10-07) a fresh child sat at 219 MB RSS
and 2.7 GB VSZ; after 20 minutes children held about 1 GB RSS and 4.8-5.6 GB of the 5.5 GiB cap (one child's map: 2.2 GB
`[anon:mimalloc]`, 1.9 GB reserved `---p`, 1.5 GB `rw-p`). The RSS recycle (25% of the cap) never fired, while libxml2's
`malloc` — not the Rust allocator, so guard 3 never saw it — failed: post-processing's XPath over `ltx:XMath` returned
nothing, logging a hundred `Error:post:xpath` "no libxml2 error detail" and ending in `Fatal:TooManyErrors` on large
papers that convert cleanly in a fresh process (113 of run 336's first 253 TooManyErrors; math0005012, 1209.0448,
1310.4783; `VmPeak` 3.6 GB for 1106.3786 on both 62zt and 62zu). So a harness child also **recycles when its `VmSize`
comes within 1.5 GiB of its `RLIMIT_AS`, or passes 3 GiB** (`cortex_worker.rs` `address_space_near_cap`,
`RECYCLE_ADDRESS_SPACE_KB`, after the result is returned; the log line names which recycle fired). The 3 GiB rule (67o,
user ruling 2026-10-15) keeps heavy papers out of grown children: 53 of run 353's 59 Error→Fatal `oom:alloc_failed`
papers convert cleanly in a fresh process under the same cap (hep-ph0006359, math0111204, math0305067). A child whose cap leaves less than that headroom above a fresh worker's 2.7 GB
(a cap below about 4.1 GiB) turns the check off at start and says so, rather than recycling after every paper.
Standalone and pooled workers, which nothing respawns, never take it.

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

## Host TeX Live for arXiv coverage

A worker resolves every `.sty`/`.cls`/`.tex` an arXiv paper loads through the host's libkpathsea tree — on Debian/Ubuntu
the distro TeX Live in `/usr/share/texlive` (+ `/usr/share/texmf`), whatever `PATH`'s `kpsewhich` says. A package missing
from that tree fails only on the fleet: the gates and local A/Bs read the vendor TL 2025 tree, so they cannot see it
(run 349: `{fmffile}` 298 and `\Tree` 302 papers were such fleet-only errors).

`texlive-full` is **not** needed. Its TeX collections are, minus documentation, command-line tools and ConTeXt:

```bash
sudo apt install --no-install-recommends \
  texlive texlive-base texlive-binaries texlive-latex-base texlive-latex-recommended texlive-latex-extra \
  texlive-fonts-recommended texlive-fonts-extra texlive-fonts-extra-links texlive-font-utils \
  texlive-bibtex-extra texlive-extra-utils texlive-formats-extra texlive-games texlive-humanities \
  texlive-lang-all texlive-luatex texlive-metapost texlive-music texlive-pictures texlive-plain-generic \
  texlive-pstricks texlive-publishers texlive-science texlive-xetex \
  feynmf
```

- `feynmf` is its own Debian package (feynmf.sty/feynmp.sty); `texlive-metapost` only *Recommends* it, so
  `--no-install-recommends` skips it unless it is named.
- `texlive-humanities` carries qtree, `texlive-metapost` mfpic/feynmp-auto, `texlive-music` musixtex; these were the
  collections the cortex host lacked until 2026-10-14.
- Optional: `asymptote` (asymptote.sty, asycolors.sty, ocg.sty for papers that `\usepackage{asymptote}`; pulls the
  `asy` program and 6 libraries).
- What `texlive-full` adds beyond this list ships no LaTeX input arXiv papers load: the nine `texlive-*-doc` packages,
  tools (`biber`, `latexmk`, `chktex`, `lacheck`, `latexdiff`, `psutils`, `purifyeps`, `dvidvi`, `fragmaster`,
  `lcdf-typetools`, `prerex`, `vprerex`), `context` (ConTeXt, not LaTeX) and `texinfo` (texinfo.tex).

Installing collections leaves the live dumps valid: their `# source` stamps name latex.ltx and expl3-code.tex, which
these packages do not touch. Keep the distro TL version uniform (`apt-get -s install …` must show 0 upgraded).

To check a host, compare its TeX input files with a full TeX Live (2026-10-14 on the cortex host, after the list
above: 250 of 17,402 file names missing — ConTeXt ~190, texinfo 24, the arphic CJK `.fd` files, asymptote's 3, and a
handful newer than the distro snapshot):

```bash
find /usr/local/texlive/2025/texmf-dist/tex -type f \( -name '*.sty' -o -name '*.cls' -o -name '*.tex' -o -name '*.def' \) \
  -printf '%f\n' | LC_ALL=C sort -u > /tmp/full.txt
find /usr/share/texlive/texmf-dist/tex /usr/share/texmf/tex -type f \( -name '*.sty' -o -name '*.cls' -o -name '*.tex' \
  -o -name '*.def' \) -printf '%f\n' | LC_ALL=C sort -u > /tmp/host.txt
LC_ALL=C comm -23 /tmp/full.txt /tmp/host.txt
```

## Exit-code reference (per child)

| Code / signal | Meaning | Source | Result the harness returns |
| --- | --- | --- | --- |
| `0` | success / warnings, or a recycle | normal | the worker's own |
| `124` | wall-clock timeout | `Watchdog` | `Fatal:timeout:wallclock` (retryable) |
| `137` | resident-memory ceiling | `Watchdog` RSS guard | `Fatal:oom:rss` |
| `137` | allocation failure (incl. `RLIMIT_AS` `ENOMEM`) | `custom_alloc_error_hook` | `Fatal:oom:alloc_failed` |
| SIGTERM | memory-pressure governor shed | pericortex governor | `Fatal:oom:shed` (retryable; `Fatal:crash:SIGTERM`, final, with the governor off) |
| SIGSEGV, SIGBUS, SIGILL, SIGFPE, SIGABRT | crash | — | `Fatal:crash:SIG<NAME>` (SIGSEGV then SIGABRT: a stack overflow) |
| SIGKILL / no recorded cause | kernel OOM killer, the harness's unresponsive-worker watchdog, an operator, an `exit` inside a library | — | `Fatal:crash:unrecorded` (retryable; the harness log has the exit status) |

After [`MAX_CONSECUTIVE_PANICS`](../../latexml_oxide/bin/cortex_worker.rs) caught panics a
worker returns the last paper's `Fatal:panic:caught` result and then exits `70`; it used to
`exit(70)` before returning it, leaving that task to the lease reaper.

## Exit reports: the result of a child that dies mid-conversion

A child that dies mid-conversion returns nothing itself, so its task used to wait for the
dispatcher's lease reaper, be re-leased, and after the retry budget end as
`Fatal:cortex:never_completed_with_retries` — the cause lost. The harness now returns a
`Status:conversion:3` result for it, through the same sink path as any result
(`latexml_oxide/bin/cortex_worker/exit_report.rs`):

* The harness makes a fresh, randomly named `$TMPDIR/cortex_harness_*/` holding a
  `harness.lock` for its life, and passes it to each child (`--exit-report-dir`, internal) once
  its reporter thread runs. It never adopts an existing directory: in the Docker image the
  harness is always PID 1 and `TMPDIR` a host mount, so a pid-named directory would hand a new
  harness the stale claims of a killed one. A child keeps `child-<pid>-<nonce>.claim` there (the nonce keeps a reused pid off a dead
  child's unread claim) under an exclusive
  `flock` for its whole life, naming its worker identity and, while a conversion runs, the task
  id; beside it `child-<pid>-<nonce>.log` mirrors the conversion log as it is written
  (`logger::set_log_mirror`, emptied per task, capped at 8 MiB).
* A child that knows why it is exiting appends a `cause` line first, without allocating: the
  watchdog's pre-exit hook (timeout / RSS), the allocation-failure hook, and a handler for the
  fatal signals (which then chains to the handler it replaced — Rust's guard-page handler — and
  dies of the signal).
* The kernel drops the lock however the process ends. The harness's `exit-reporter` thread polls
  the claims every 250 ms; a claim it can lock is a dead child's, and if it names a task the
  reporter sends `[identity, service, taskid, result.zip]` to the sink — under the child's own
  identity, which the dispatcher uses to attribute the result to its dispatch attempt — with
  the partial log, the cause's `Fatal:` line and `Status:conversion:3`.
* A SIGTERM or unrecorded death while the harness itself is shutting down is not reported (the
  dispatcher re-leases the task). A clean exit removes its claim. The harness removes its
  directory on shutdown and, when it starts, every `cortex_harness_*` whose lock nobody holds,
  without reporting from it (that fleet is gone; the dispatcher re-leases its tasks).
* A cause is recorded only by the claim's owner process (not by a subprocess between `fork` and
  `exec`, which inherits the handlers), and the allocation-failure hook records it before
  anything that could allocate.
* **Retry, then cause** (user ruling 2026-10-12). A watchdog timeout, a governor shed and an
  unrecorded death may not repeat on another try: their report carries CorTeX's
  `Info:cortex:retryable <category>:<what>` tag, and the dispatcher returns the task to its retry
  budget instead of recording it, reporting the last such cause (not
  `never_completed_with_retries`) once the budget is spent. A crash by a signal (a stack overflow
  included) and the memory limits (`oom:rss`, `oom:alloc_failed`) are final on their first
  occurrence — except an `oom:alloc_failed` in a child whose address space at the paper's start (the claim's
  `space <KiB>` line, written by `begin_task`) was already past a fresh worker's 2.7 GB: that is the child's history,
  so it is retryable (`exit_report::retryable`, user ruling 2026-10-15). A document's own lines shaped `<severity>:cortex:…` (a `\message`; any severity, as
  CorTeX lowercases it and files an unknown one as Info) are indented by a space in every log the
  worker returns, so they cannot pose as the tag.
* **Deploy order:** deploy the CorTeX dispatcher at `fix/late-results-and-exit-causes` (round 2 or
  later) **before** a fleet built from this commit. An older dispatcher ignores the tag and records
  every retryable cause as a final Fatal.

Not covered: a death between a conversion's end and its result reaching the sink (the claim is
idle by then) — the lease reaper is still the net there. Guards: the unit tests in
`exit_report.rs`, and `latexml_oxide/tests/cortex_worker_exit_reports.rs`, which runs a
one-worker harness against an in-test dispatcher for each cause (the faults come from
`CORTEX_WORKER_TEST_FAULT`, compiled into debug-assertion builds only; CI never enables the
`cortex` feature, so run locally with
`cargo test -p latexml --features cortex --test cortex_worker_exit_reports`).
