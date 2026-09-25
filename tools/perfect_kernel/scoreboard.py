#!/usr/bin/env python3
r"""Perfect-kernel scoreboard: one row of corpus metrics per sweep.

    scoreboard.py [FIRST [LAST]]      (sweep numbers; default 105..999)

Reads the per-sweep verdict TSVs, which every sweep keeps even after its per-document outputs
are compacted:
  ~/data/perfect_kernel_s<N>/sweep_verdicts.tsv       bundle name status exit errors fatals · secs
  ~/data/perfect_kernel_s<N>/validate_verdicts.tsv    bundle name jing-errors
  ~/data/perfect_kernel_s<N>_html/s3_verdicts.tsv     bundle name recall% found total missing …

Columns: docs; clean = status 0-1 (no Error, no Fatal); fatal = status 3; timeout = status 124
or 137; errors = the sum of Error lines; valid = XMLs with 0 jing errors; scored = docs with an
HTML recall; recall mean / median / share >= 95 %; missing = distinct PDF words missing, summed.
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


def row(n):
    d = f'{DATA}/perfect_kernel_s{n}'
    sv = f'{d}/sweep_verdicts.tsv'
    if not os.path.exists(sv):
        return None
    st = [l.rstrip('\n').split('\t') for l in open(sv) if l.strip()]
    clean = sum(1 for r in st if r[2] in ('0', '1'))
    fatal = sum(1 for r in st if r[2] == '3')
    tout = sum(1 for r in st if r[2] in ('124', '137'))
    errs = sum(int(r[4]) for r in st if r[4].isdigit())
    vv = f'{d}/validate_verdicts.tsv'
    valid = (sum(1 for l in open(vv) if l.rstrip('\n').split('\t')[2] == '0')
             if os.path.exists(vv) else '-')
    rec, miss = [], 0
    s3 = f'{d}_html/s3_verdicts.tsv'
    if os.path.exists(s3):
        for l in open(s3):
            f = l.rstrip('\n').split('\t')
            if len(f) > 5 and num(f[2]) is not None:
                rec.append(num(f[2]))
                miss += int(f[5]) if f[5].isdigit() else 0
    if rec:
        tail = (len(rec), f'{statistics.mean(rec):.2f}', f'{statistics.median(rec):.1f}',
                f'{100 * sum(1 for x in rec if x >= 95) / len(rec):.1f}', miss)
    else:
        tail = ('-',) * 5
    return (n, len(st), clean, fatal, tout, errs, valid) + tail


def main():
    first = int(sys.argv[1]) if len(sys.argv) > 1 else 105
    last = int(sys.argv[2]) if len(sys.argv) > 2 else 999
    print('sweep\tdocs\tclean\tfatal\ttimeout\terrors\tvalid\tscored\trecall_mean\tmedian\t'
          '%>=95\tmissing')
    for n in range(first, last + 1):
        r = row(n)
        if r:
            print('\t'.join(str(x) for x in r))


if __name__ == '__main__':
    main()
