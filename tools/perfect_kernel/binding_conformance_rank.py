#!/usr/bin/env python3
"""Rank a binding_conformance.sh run (K13 stage 2): <outdir>/rows/*.tsv and status.tsv, weighted by
weights.tsv, into conformance.tsv (findings by weight, then severity) and summary.tsv (one row per
package); print the packages whose audit failed (non-zero exit, no header, or rows missing).
    usage: binding_conformance_rank.py <outdir>"""
import os, sys

out = sys.argv[1]
def table(path):
    return dict(l.split('\t', 1) for l in open(path).read().split('\n') if l.strip())
weights = table(f'{out}/weights.tsv')
status = table(f'{out}/status.tsv')
rank = {'HIGH': 0, 'MEDIUM': 1, 'LOW': 2}
rows, summary, failed = [], [], []
for package in sorted(status):
    path = f'{out}/rows/{package}.tsv'
    # Split on newlines only: a field never holds one (binding_audit writes `^^M` for a CR).
    lines = [l for l in open(path).read().split('\n') if l] if os.path.exists(path) else []
    head = [l.split('\t') for l in lines if l.startswith('#\t')]
    found = [l.split('\t') for l in lines if not l.startswith('#\t')]
    weight = weights.get(package, '0').strip()
    if status[package].strip() != '0' or not head:
        failed.append(f'{package}: exit {status[package].strip()}, {"no header" if not head else "header"}')
        summary.append([weight, package, status[package].strip()])
        continue
    h = head[0]
    # h: '#', package, defined, passthrough, conformant, findings, raw_errors, binding_errors, raw_warnings, binding_warnings
    if int(h[5]) != len(found) or any(len(r) != 10 for r in found):
        failed.append(f'{package}: header says {h[5]} findings, {len(found)} rows')
    summary.append([weight, package, '0'] + h[2:])
    for r in found:
        rows.append([weight] + r + [f'{h[6]}/{h[7]}'])
rows.sort(key=lambda r: (-int(r[0]), rank.get(r[3], 3), r[1], r[2]))
summary.sort(key=lambda r: (-int(r[0]), r[1]))
with open(f'{out}/conformance.tsv', 'w') as f:
    f.write('weight\tpackage\tcs\tseverity\tclasses\traw\tbinding\traw_chain\tbinding_chain\traw_notes\tbinding_notes\tsession_errors\n')
    for r in rows:
        f.write('\t'.join(r) + '\n')
with open(f'{out}/summary.tsv', 'w') as f:
    f.write('weight\tpackage\texit\tdefined\tpassthrough\tconformant\tfindings\traw_errors\tbinding_errors\traw_warnings\tbinding_warnings\n')
    for r in summary:
        f.write('\t'.join(r) + '\n')
by = {}
for r in rows:
    by[r[3]] = by.get(r[3], 0) + 1
print(f'packages {len(status)}; findings {len(rows)} {by}')
for line in failed:
    print('FAILED', line)
sys.exit(1 if failed else 0)
