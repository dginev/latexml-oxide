# Docker Images for latexml-oxide

This directory contains Dockerfiles for building and deploying latexml-oxide services.

## Images

### cortex-worker

**File:** `cortex-worker.dockerfile`

A CorTeX worker that converts TeX documents to HTML using latexml-oxide. Implements the [pericortex](https://github.com/dginev/cortex-peripherals) `Worker` trait for integration with the [CorTeX](https://github.com/dginev/cortex) distributed processing framework.

**Build:**
```bash
docker build -f docker/cortex-worker.dockerfile -t cortex-worker .
```

**Run (worker mode):**
```bash
# Connect to a CorTeX dispatcher
docker run cortex-worker \
  --source-address tcp://dispatcher:51695 \
  --sink-address tcp://dispatcher:51696 \
  --pool-size 4 \
  --profile ar5iv
```

**Run (standalone mode):**
```bash
# Single ZIP-to-ZIP conversion without a dispatcher
docker run -v /data:/data cortex-worker \
  --standalone \
  --input /data/paper.zip \
  --output /data/result.zip
```

**Profiles:**
- `ar5iv` (default) — matches the ar5iv production pipeline: `--preload=ar5iv.sty --pmml --mathtex --noinvisibletimes --nodefaultresources`
- `generic` — minimal HTML5 conversion without ar5iv-specific preloads

**System requirements:**
- ~2GB disk for the Docker image (TeX Live + Rust binary)
- ~500MB RAM per worker thread for typical arXiv papers
- ZMQ connectivity to CorTeX dispatcher (worker mode)
