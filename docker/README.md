# Docker Images for latexml-oxide

Both published images build from a **single, unified Dockerfile** at the repo
root (`../Dockerfile`), selected with `--target`. Dependencies (TeX Live,
graphics tools, build toolchain) are declared once in shared `texbase` +
`toolchain` stages; only the per-binary build command and entrypoint differ.
This directory holds the worker entrypoint script.

`.github/workflows/docker.yml` builds + pushes both to GHCR on `release: published`
(CLI multi-arch amd64+arm64; worker amd64).

## `cli` — general-purpose CLI · `ghcr.io/dginev/latexml-oxide`

The plain `latexml_oxide` CLI plus a reproducible TeX Live + graphics
environment. The image builds its own binary and **embeds** the kernel dumps
(self-contained: nothing read from its own resources at convert time). No local
TeX Live needed.

```bash
# build (default target)
docker build --target cli -t ghcr.io/dginev/latexml-oxide .

# convert — bind-mount your document tree
docker run --rm -v "$PWD:/work" ghcr.io/dginev/latexml-oxide paper.tex
```

## `worker` — CorTeX fleet · `ghcr.io/dginev/latexml-oxide/cortex-worker`

**Entrypoint:** `cortex-worker-entrypoint.sh`

A turnkey CorTeX worker fleet that converts TeX to HTML with latexml-oxide. It
implements the [pericortex](https://github.com/dginev/cortex-peripherals)
`Worker` trait for the [CorTeX](https://github.com/dginev/cortex) distributed
pipeline, and is the **Rust counterpart of the legacy Perl fleet image**
([`LaTeXML-Plugin-CorTeX/Dockerfile`](https://github.com/dginev/LaTeXML-Plugin-CorTeX/blob/master/Dockerfile)):
same dispatcher, ZMQ ports (`51695`/`51696`), and result-archive contract —
different engine and a different service name (`oxidized_tex_to_html` vs the
Perl `tex_to_html`). One dispatcher can run both fleets at once; it leases by
`service_id`.

**Build:**
```bash
export HOSTTIME=$(date -Iminute)
docker build --target worker --build-arg HOSTTIME=$HOSTTIME -t cortex-worker:latest .
```

**Run the fleet (turnkey — pass just the dispatcher host; `--harness` self-supervises and auto-sizes the box):**
```bash
# local dispatcher (loopback, skips the Docker NAT — run on the dispatcher host):
docker run --network host -v /opt/cortex-scratch:/opt/cortex-scratch --hostname=$(hostname) cortex-worker 127.0.0.1

# remote dispatcher:
docker run -v /opt/cortex-scratch:/opt/cortex-scratch --hostname=$(hostname) cortex-worker DISPATCHER_IP

# positional args:  <dispatcher-host> [ventilator-port=51695] [sink-port=51696] [service=oxidized_tex_to_html]
```

**Tuning (env vars):**
- `PROFILE` — `ar5iv` (default, the production pipeline) | `generic`
- `WORKERS` — pin the fleet size per host. Battle-hardened sweet spot is **physical cores + 1/8**
  (e.g. 72 on a 64-physical box). Unset = the harness CPU-derived default (sizes to *logical* cores,
  a deliberate over-commit). Per-child memory caps, the recycle threshold, the memory-pressure
  governor, and the per-document timeout stay at the binary's validated defaults — see
  [`docs/performance/CORTEX_WORKER_HARNESS.md`](../docs/performance/CORTEX_WORKER_HARNESS.md).

**Advanced — standalone (pass cortex_worker flags straight through):**
```bash
# Single ZIP-to-ZIP conversion without a dispatcher (first arg starts with `-` → flags verbatim):
docker run -v /data:/data cortex-worker --standalone --input /data/paper.zip --output /data/result.zip
```

**System requirements:**
- ~6 GB disk for the image (full arXiv-capable TeX Live + the Rust binary).
- ~1 GB RAM per worker for typical arXiv papers (a heavy paper trips the per-child ceiling and is
  shed/respawned — see the harness doc); size `--memory`/`--cpus` and `WORKERS` per host.
- Disk-backed scratch on a SEPARATE physical disk from the OS, bind-mounted as `TMPDIR`:
  `-v /opt/cortex-scratch:/opt/cortex-scratch`. Do NOT use a ramdisk (`/dev/shm`) — exhaustion under a
  large fleet truncates inputs → empty 0-byte results (CorTeX KNOWN_ISSUES D-18).
- ZMQ connectivity to the CorTeX dispatcher (fleet mode).
