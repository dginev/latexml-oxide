#!/usr/bin/env python3
r"""Perfect-kernel scoreboard: one row of corpus metrics per sweep.

    scoreboard.py [--scope in|all] [FIRST [LAST]]      (sweep numbers; default 105..999)

Scope (user, 2026-09-29): the program works on the manuals that compile cleanly with at least one
of pdflatex, lualatex or xelatex — `oracle.sh`'s verdict, exit 0 and no `!` line
(`~/data/perfect_kernel/oracle_verdicts.tsv`; rerun it when TeX Live changes). The quality table is
that set by default (`--scope all`: the whole corpus, the pre-2026-09-29 view); the manuals no engine
compiles follow as crash canaries — docs, fatal, timeout, other abnormal exits, cpu_h, over 120 s —
watched for crashes and timeouts only.

Reads the per-sweep verdict TSVs, which every sweep keeps even after its per-document outputs
are compacted:
  ~/data/perfect_kernel_s<N>/sweep_verdicts.tsv       bundle name status exit errors fatals · secs
  ~/data/perfect_kernel_s<N>/validate_verdicts.tsv    bundle name jing-errors
  ~/data/perfect_kernel_s<N>_html/s3_verdicts.tsv     bundle name recall% found total missing …

Columns: docs; clean = status 0-1 (no Error, no Fatal); fatal = status 3; timeout = status 124
or 137; errors = the sum of Error lines; valid = XMLs with 0 jing errors; scored = docs with an
HTML recall; recall mean / median / share >= 95 %; missing = distinct PDF words missing, summed;
then performance (per-document wall seconds of the conversion, core XML): cpu_h = their sum in
hours, p90 / p99, and the count over 60 s and over 120 s (the 180 s ceiling's approach).
The roadmap's gates read this table (docs/PERFECT_KERNEL.md "Roadmap").
"""
import os
import statistics
import sys

DATA = os.path.expanduser('~/data')


def num(x):
    try:
        return float(x)
    except ValueError:
        return None


ORACLE = f'{DATA}/perfect_kernel/oracle_verdicts.tsv'


def oracle_clean():
    """(bundle, name) of the manuals some engine compiles cleanly: exit 0, no `!` line."""
    keep = set()
    for l in open(ORACLE):
        f = l.rstrip('\n').split('\t')
        if len(f) > 4 and f[3] == '0' and f[4] == '0':
            keep.add((f[0], f[1]))
    return keep


def rows_of(path, keep):
    rows = [l.rstrip('\n').split('\t') for l in open(path) if l.strip()]
    return [r for r in rows if keep is None or (r[0], r[1]) in keep]


def canary_row(n, drop):
    """The manuals out of scope: crashes and time only."""
    sv = f'{DATA}/perfect_kernel_s{n}/sweep_verdicts.tsv'
    if not os.path.exists(sv):
        return None
    st = [r for r in rows_of(sv, None) if (r[0], r[1]) in drop]
    fatal = sum(1 for r in st if r[2] == '3')
    tout = sum(1 for r in st if r[2] in ('124', '137'))
    other = sum(1 for r in st if r[2] not in ('0', '1', '2', '3', '124', '137'))
    secs = [num(r[7]) for r in st if len(r) > 7 and num(r[7]) is not None]
    return (n, len(st), fatal, tout, other, f'{sum(secs) / 3600:.2f}', sum(1 for x in secs if x > 120))


def row(n, keep=None):
    d = f'{DATA}/perfect_kernel_s{n}'
    sv = f'{d}/sweep_verdicts.tsv'
    if not os.path.exists(sv):
        return None
    st = rows_of(sv, keep)
    clean = sum(1 for r in st if r[2] in ('0', '1'))
    fatal = sum(1 for r in st if r[2] == '3')
    tout = sum(1 for r in st if r[2] in ('124', '137'))
    errs = sum(int(r[4]) for r in st if r[4].isdigit())
    secs = sorted(num(r[7]) for r in st if len(r) > 7 and num(r[7]) is not None)
    q = lambda x: secs[int(x * (len(secs) - 1))] if secs else 0.0
    perf = (f'{sum(secs) / 3600:.2f}', f'{q(0.9):.1f}', f'{q(0.99):.1f}',
            sum(1 for x in secs if x > 60), sum(1 for x in secs if x > 120))
    vv = f'{d}/validate_verdicts.tsv'
    valid = (sum(1 for r in rows_of(vv, keep) if r[2] == '0')
             if os.path.exists(vv) else '-')
    rec, miss = [], 0
    s3 = f'{d}_html/s3_verdicts.tsv'
    if os.path.exists(s3):
        for f in rows_of(s3, keep):
            if len(f) > 5 and num(f[2]) is not None:
                rec.append(num(f[2]))
                miss += int(f[5]) if f[5].isdigit() else 0
    if rec:
        tail = (len(rec), f'{statistics.mean(rec):.2f}', f'{statistics.median(rec):.1f}',
                f'{100 * sum(1 for x in rec if x >= 95) / len(rec):.1f}', miss)
    else:
        tail = ('-',) * 5
    return (n, len(st), clean, fatal, tout, errs, valid) + tail + perf


def main():
    args = sys.argv[1:]
    scope = 'in'
    if args[:1] == ['--scope']:
        scope, args = args[1], args[2:]
    first = int(args[0]) if len(args) > 0 else 105
    last = int(args[1]) if len(args) > 1 else 999
    keep = oracle_clean() if scope == 'in' else None
    print(f'# quality: {"manuals some engine compiles cleanly" if keep else "all manuals"}')
    print('sweep\tdocs\tclean\tfatal\ttimeout\terrors\tvalid\tscored\trecall_mean\tmedian\t'
          '%>=95\tmissing\tcpu_h\tp90_s\tp99_s\t>60s\t>120s')
    for n in range(first, last + 1):
        r = row(n, keep)
        if r:
            print('\t'.join(str(x) for x in r))
    if keep is None:
        return
    every = {(f[0], f[1]) for f in rows_of(ORACLE, None)}
    print('# crash canaries: manuals no engine compiles cleanly (crashes and time only)')
    print('sweep\tdocs\tfatal\ttimeout\tother\tcpu_h\t>120s')
    for n in range(first, last + 1):
        r = canary_row(n, every - keep)
        if r:
            print('\t'.join(str(x) for x in r))


if __name__ == '__main__':
    main()
