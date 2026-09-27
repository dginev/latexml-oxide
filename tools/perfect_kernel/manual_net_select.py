#!/usr/bin/env python3
"""Select the fixed manual regression net (KERNEL_CAPABILITIES K17).

    manual_net_select.py <sweep-dir> <corpus.tsv> [n=1000] > tools/perfect_kernel/manual_net.tsv

Greedy coverage in two stages. First, what each manual loads: every binding it loads (a
`(Loading siunitx_sty.rs... )` log line; also `.ldf`, `.def`, fontmaps) and every raw file it reads
as definitions (`(Processing definitions <path>...`: a raw `.sty`, `.cls`, `.def`, …), by basename.
When loads are covered, what each manual produces: every element name of its core XML (`svg:path`
included) and every `class` value, with the user-named families (`ltx_theorem_*`, `ltx_lst_*`,
`ltx_float_*`) counted once each. The next pick is the manual that adds the most features not yet
covered per square root of its run time, until nothing new is covered (n caps it). Candidates ended
with status 0-2 (no Fatal, timeout or kill) in under 45 s, so the heaviest manuals are outside the
net; at most two come from one bundle. Ties break by name: the same sweep always yields the same
net. The output rows are corpus.tsv rows, which sweep.sh reads as they are.
"""
import math
import os
import re
import sys

BASE = {'TeX.pool', 'LaTeX.pool', 'latexml_sty.rs', 'textcomp_sty.rs'}
FAMILIES = ('ltx_theorem_', 'ltx_lst_', 'ltx_float_')
MAX_SECS = 45.0
PER_BUNDLE = 2


def produced_features(xml):
  body = re.sub(r'<\?[^>]*\?>', '', xml)  # processing instructions carry `class="…"` too
  features = set(re.findall(r'<([\w:]+)[\s/>]', body))
  for classes in re.findall(r' class="([^"]*)"', body):
    for value in classes.split():
      family = next((f for f in FAMILIES if value.startswith(f)), None)
      features.add(family + '*' if family else value)
  return features


def main():
  sweep, corpus = sys.argv[1], sys.argv[2]
  n = int(sys.argv[3]) if len(sys.argv) > 3 else 1000
  rows = {}
  for line in open(corpus):
    f = line.rstrip('\n').split('\t')
    rows[(f[0], os.path.basename(f[1])[:-4])] = line.rstrip('\n')
  candidates = []
  for line in open(os.path.join(sweep, 'sweep_verdicts.tsv')):
    bundle, name, status, _exit, _err, _fatals, _warn, secs = line.rstrip('\n').split('\t')[:8]
    if (bundle, name) not in rows or status not in ('0', '1', '2') or float(secs) > MAX_SECS:
      continue
    base = os.path.join(sweep, bundle, name, name)
    try:
      log = open(base + '.log', errors='replace').read()
    except OSError:
      continue
    loads = set(re.findall(r'\(Loading (\S+?)\.\.\.', log))
    loads |= {os.path.basename(p) for p in re.findall(r'\(Processing definitions (\S+?)\.\.\.', log)}
    loads -= BASE
    try:
      produced = produced_features(open(base + '.xml', errors='replace').read())
    except OSError:
      produced = set()
    candidates.append((bundle, name, float(secs), loads, produced))
  chosen, per_bundle = [], {}
  for stage in (3, 4):
    covered = set().union(*(c[stage] for c in chosen)) if chosen else set()
    pool = [c for c in candidates if c not in chosen]
    while len(chosen) < n and pool:
      def gain(c):
        return len(c[stage] - covered) / math.sqrt(c[2] + 1.0)
      best = max(pool, key=lambda c: (gain(c), c[0], c[1]))
      if not best[stage] - covered:
        break
      pool.remove(best)
      if per_bundle.get(best[0], 0) >= PER_BUNDLE:
        continue
      per_bundle[best[0]] = per_bundle.get(best[0], 0) + 1
      covered |= best[stage]
      chosen.append(best)
    reachable = set().union(*(c[stage] for c in candidates)) if candidates else set()
    print(f'# stage {stage - 2}: {len(chosen)} manuals; {len(covered)} of the {len(reachable)} '
          f'features of the candidates covered', file=sys.stderr)
  for c in sorted(chosen, key=lambda c: (c[0], c[1])):
    print(rows[(c[0], c[1])])
  print(f'# {len(candidates)} candidates; {sum(c[2] for c in chosen):.0f} s serial', file=sys.stderr)


if __name__ == '__main__':
  main()
