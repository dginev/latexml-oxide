# Thermals and CPU/memory budget on the dev laptop

Written 2026-09-02 after the laptop hit sustained thermal throttling. Consult
this before starting any parallel workload (corpus sweeps, `cargo nextest`,
oracle/validate runs), and especially before starting two of them at once.

## The machine

- Intel i7-12800H, 20 threads (6 P-cores + 8 E-cores), Tjmax 100 °C.
- 31 GB RAM, 8 GB swap. No headroom beyond that.
- Sustained all-core load pins the package at 95–96 °C and the kernel
  throttles clocks to ~3.0–3.8 GHz (vs 4.7 GHz max). The fan gets very loud.
  This is the CPU protecting itself, not a hardware fault.
- The `SEN2` thermal zone reads a constant 128 °C. That is a sentinel for a
  missing sensor. Ignore it; trust `x86_pkg_temp`.

## What happened on 2026-09-02

Two workloads were running simultaneously from separate Claude sessions:

| Workload | Parallelism | Effective cost |
|---|---|---|
| `tools/perfect_kernel/sweep.sh` | `xargs -P 10`, each doc up to 6 GB (`--max-memory=6144`) | 10+ cores, up to 60 GB memory ceiling |
| `cargo nextest run -j 8 --workspace` | 8 test binaries; cluster-regression tests fork their own `latexml_oxide` | 8+ cores |

Result: 22–24 `latexml_oxide` processes at 100 % CPU on 20 threads, load
average 15–17, swap 100 % full (8191/8191 MB), 3 GB RAM available, and
~90 package-throttle events per second.

The `-j` flag on nextest does **not** bound this. Most of the CPU and all of
the memory pressure came from the sweep, which nextest knows nothing about.

## Rules of thumb

1. **Do not overlap a sweep with a test run.** Let one finish first. A
   10-wide sweep alone already fills the machine.
2. **Sweeps, oracle, validate:** `sweep.sh`, `oracle.sh` and `validate.sh`
   all read `JOBS` (default 8 since 2026-09-02). On this laptop:
   - alone: `JOBS=8` is the practical ceiling. `JOBS=12` will throttle and
     can exhaust swap because 12 × 6 GB > 31 GB + 8 GB.
   - alongside anything else: `JOBS=4`.
3. **nextest:** `-j 8` is fine alone. Use `-j 4` if a sweep is running,
   and remember the `*_cluster_regressions` tests spawn extra converters.
4. **Memory, not just CPU, is the binding constraint.** Count
   `parallel jobs × --max-memory`. Keep it under ~24 GB so the OS and the
   editor/rust-analyzer (which alone uses ~4 GB) have room.

## How to check

```sh
# package temp (millidegrees) and whether throttling is active right now
cat /sys/class/thermal/thermal_zone*/type /sys/class/thermal/thermal_zone*/temp
a=$(cat /sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count); sleep 5
echo $(( $(cat /sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count) - a )) throttle events / 5s

# who is burning CPU, and how many converters are alive
ps -eo pcpu,pmem,etime,comm --sort=-pcpu | head
pgrep -fc latexml_oxide

# memory and swap
free -m
```

A nonzero throttle delta over 5 s, or swap near full, means back off: reduce
`JOBS` on the next invocation, or wait for the current run to finish. Do not
kill a running sweep mid-way unless memory is actually exhausting; it does
not checkpoint, so a kill loses the whole run.
