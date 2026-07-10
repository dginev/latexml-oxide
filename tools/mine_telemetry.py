#!/usr/bin/env python3
"""Corpus-wide telemetry miner for a completed cortex run (companion to
tools/telemetry_dashboard.py and tools/perf_phase_summary.py).

    tools/mine_telemetry.py extract   # parallel-read every result ZIP's
                                      # telemetry.json -> a JSONL cache
    tools/mine_telemetry.py analyze   # phase budget, wall/RSS tail, math
                                      # over-parse, slow-tail drivers, fatal profile

Reads /data/arxmliv/{2605,2606}/*/oxidized_tex_to_html.sandbox-{13,14}.zip by
default (edit find_zips for other runs). Produced the 2026-07-10 phase budget in
docs/ARXIV_PERFORMANCE.md and the BP-1..BP-6 plan in docs/SYNC_STATUS.md.
"""
import os, sys, json, zipfile, glob
from multiprocessing import Pool
from collections import defaultdict

PHASES = ["bootstrap","digest","build","rewrite","math_parse","post_xml_parse",
"post_scan","bibliography","crossref","graphics","math_images","mathml_pres",
"mathml_cont","split","xslt","html5_fixups","serialize"]

def find_zips():
    zs = []
    for corpus, sb in (("2605","sandbox-13"),("2606","sandbox-14")):
        base = f"/data/arxmliv/{corpus}"
        for root, _, files in os.walk(base):
            f = f"oxidized_tex_to_html.{sb}.zip"
            if f in files:
                zs.append((corpus, os.path.join(root, f)))
    return zs

def read_one(arg):
    corpus, path = arg
    try:
        with zipfile.ZipFile(path) as z:
            d = json.loads(z.read("telemetry.json"))
        d["_corpus"] = corpus
        d["_name"] = os.path.basename(os.path.dirname(path))
        return d
    except Exception:
        return None

def extract(jsonl):
    zs = find_zips()
    print(f"found {len(zs)} result zips; reading telemetry.json in parallel...", file=sys.stderr)
    with Pool(24) as p, open(jsonl,"w") as out:
        n=0
        for rec in p.imap_unordered(read_one, zs, chunksize=64):
            if rec is not None:
                out.write(json.dumps(rec)+"\n"); n+=1
    print(f"wrote {n} records", file=sys.stderr)

def pct(sorted_vals, p):
    if not sorted_vals: return 0
    import math
    k = max(1, math.ceil(p/100*len(sorted_vals)))
    return sorted_vals[k-1]

def analyze(jsonl):
    recs=[json.loads(l) for l in open(jsonl)]
    n=len(recs)
    print(f"\n=== {n} records ===")

    # outcome bucket
    def bucket(r):
        if r.get("fatal_errors",0)>0 or "fatal" in r.get("category","") or r.get("exit_code",0)>=3: return "fatal"
        if r.get("errors",0)>0 or r.get("category")=="conversion_error" or r.get("exit_code",0)==2: return "error"
        if r.get("warnings",0)>0: return "warning"
        return "no_problem"
    oc=defaultdict(int)
    for r in recs: oc[bucket(r)]+=1
    print("outcomes:", dict(oc))

    # --- A. Phase time budget (share of total wall) ---
    total_wall=sum(r.get("wall_us",0) for r in recs)
    phase_tot=[0]*17
    for r in recs:
        pu=r.get("phase_us",[])
        for i in range(min(17,len(pu))): phase_tot[i]+=pu[i]
    sum_phase=sum(phase_tot)
    print(f"\n--- A. Where wall time goes (total {total_wall/1e6/3600:.1f} core-hours) ---")
    print(f"{'phase':16} {'share_of_wall':>13} {'share_of_summed_phases':>22}")
    order=sorted(range(17), key=lambda i:-phase_tot[i])
    for i in order:
        if phase_tot[i]==0: continue
        print(f"{PHASES[i]:16} {100*phase_tot[i]/total_wall:12.1f}% {100*phase_tot[i]/sum_phase:21.1f}%")
    print(f"{'(sum of phases)':16} {100*sum_phase/total_wall:12.1f}%  (rest = harness/IO/uninstrumented)")

    # --- B. Tail concentration ---
    walls=sorted((r.get("wall_us",0) for r in recs))
    tw=sum(walls)
    def tailshare(fr):
        k=int(len(walls)*fr); return 100*sum(walls[-k:])/tw if k else 0
    print(f"\n--- B. Tail concentration (wall) ---")
    print(f"median={pct(walls,50)/1e6:.2f}s  P90={pct(walls,90)/1e6:.1f}s  P99={pct(walls,99)/1e6:.1f}s  max={walls[-1]/1e6:.1f}s")
    print(f"slowest 1% of papers hold {tailshare(0.01):.0f}% of all wall; slowest 5% hold {tailshare(0.05):.0f}%")
    for thr in (30,60,120,180):
        c=sum(1 for w in walls if w>=thr*1e6)
        print(f"  papers >= {thr:3}s: {c:5} ({100*c/n:.2f}%)")

    # --- C. RSS distribution ---
    rss=sorted((r.get("max_rss_kb",0)/1024/1024 for r in recs))  # GiB
    print(f"\n--- C. Peak RSS (GiB) ---")
    print(f"median={pct(rss,50):.2f}  P90={pct(rss,90):.2f}  P99={pct(rss,99):.2f}  max={rss[-1]:.2f}")
    for thr in (2,3,4):
        c=sum(1 for x in rss if x>=thr)
        print(f"  papers >= {thr} GiB: {c:5} ({100*c/n:.2f}%)")

    # --- D. Math over-parse ---
    att=sum(r.get("math_parse_attempts",0) for r in recs)
    cnt=sum(r.get("math_parse_count",0) for r in recs)
    form=sum(r.get("formulae",0) for r in recs)
    print(f"\n--- D. Math parsing ---")
    print(f"formulae={form:,}  math_parse_count={cnt:,}  math_parse_attempts={att:,}")
    if cnt: print(f"  attempts/parse ratio = {att/cnt:.2f}x  (over-parse factor)")
    mp_share=100*phase_tot[4]/total_wall
    print(f"  math_parse is {mp_share:.1f}% of total wall")

    # --- E. Drivers of the slow tail ---
    print(f"\n--- E. Slowest 50 papers: which phase dominates? ---")
    slow=sorted(recs, key=lambda r:-r.get("wall_us",0))[:50]
    dom=defaultdict(int)
    for r in slow:
        pu=r.get("phase_us",[])
        if pu: dom[PHASES[max(range(min(17,len(pu))), key=lambda i:pu[i])]]+=1
    print("  dominant phase among slowest-50:", dict(sorted(dom.items(), key=lambda x:-x[1])))
    print("  top 8 slowest:")
    for r in slow[:8]:
        pu=r.get("phase_us",[])
        dph=PHASES[max(range(min(17,len(pu))), key=lambda i:pu[i])] if pu else "?"
        print(f"    {r['_corpus']}/{r['_name']:12} {r.get('wall_us',0)/1e6:6.1f}s rss={r.get('max_rss_kb',0)/1024/1024:.1f}G forms={r.get('formulae',0):5} dom={dph} [{bucket(r)}]")

    # --- F. Fatal vs OK profile ---
    print(f"\n--- F. Fatal vs no_problem profile ---")
    for grp in ("fatal","no_problem"):
        g=[r for r in recs if bucket(r)==grp]
        if not g: continue
        gw=sorted(r.get("wall_us",0) for r in g)
        print(f"  {grp:11} n={len(g):5} median_wall={pct(gw,50)/1e6:.2f}s mean_wall={sum(gw)/len(g)/1e6:.2f}s P99={pct(gw,99)/1e6:.1f}s")

if __name__=="__main__":
    if len(sys.argv)<2 or sys.argv[1] not in ("extract","analyze"):
        sys.exit("usage: mine_telemetry.py {extract|analyze} [telemetry.jsonl]")
    jsonl = sys.argv[2] if len(sys.argv)>2 else "telemetry.jsonl"
    (extract if sys.argv[1]=="extract" else analyze)(jsonl)
