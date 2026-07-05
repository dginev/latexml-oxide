# Docker Images for latexml-oxide

This directory contains Dockerfiles for building and deploying latexml-oxide services.

## Images

### cortex-worker

**File:** `cortex-worker.dockerfile` · **Entrypoint:** `cortex-worker-entrypoint.sh`

A turnkey CorTeX worker fleet that converts TeX documents to HTML using latexml-oxide. It implements
the [pericortex](https://github.com/dginev/cortex-peripherals) `Worker` trait for the
[CorTeX](https://github.com/dginev/cortex) distributed pipeline, and is the **Rust counterpart of the
legacy Perl fleet image** ([`LaTeXML-Plugin-CorTeX/Dockerfile`](https://github.com/dginev/LaTeXML-Plugin-CorTeX/blob/master/Dockerfile)):
same dispatcher, ZMQ ports (`51695`/`51696`), and result-archive contract — different engine and a
different service name (`oxidized_tex_to_html` vs the Perl `tex_to_html`). One dispatcher can run
both fleets at once; it leases by `service_id`.

**Build:**
```bash
export HOSTTIME=$(date -Iminute)
docker build -f docker/cortex-worker.dockerfile --build-arg HOSTTIME=$HOSTTIME -t cortex-worker:latest .
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
  [`docs/CORTEX_WORKER_HARNESS.md`](../docs/CORTEX_WORKER_HARNESS.md).

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
