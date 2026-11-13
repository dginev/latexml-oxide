#!/usr/bin/env python3
"""Compare the two runs of the manual regression net (KERNEL_CAPABILITIES K17).

    manual_net_compare.py <outdir> [--recall]

<outdir> is manual_net.sh's: A/ and B/ hold the two sweeps of manual_net.tsv (with
validate_verdicts.tsv), repros.A/ and repros.B/ the catalog runs. Per manual: status, errors,
fatals, warnings, schema validity, run time, and a byte comparison of the core XML with its word
count and Math `tex=` fingerprint (a change there is invisible to every count, 56jo). `--recall`
scores every manual whose XML changed against its PDF (pdf_recall.py, min-len 4). Per repro: its
row in each side's topic table.

It fails toward flagging (CLAUDE.md): a manual missing from either side, an incomplete repro run and
a recall that cannot be scored are regressions. The REGRESSIONS block lists what counts against a
batch: a worse status, a new Fatal, more errors, schema-valid → invalid, words down by more than 1 %,
recall down, a Math `tex=` change, a lost XML, and a repro that newly fails. CHANGED lists every
change, words up by more than 1 % (invented content) and runs slower by more than 50 % and 5 s
included; each needs a reason.
"""
import collections
import glob
import hashlib
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def net_keys():
  keys, pdfs = [], {}
  for line in open(os.path.join(HERE, 'manual_net.tsv')):
    f = line.rstrip('\n').split('\t')
    key = (f[0], os.path.basename(f[1])[:-4])
    keys.append(key)
    pdfs[key] = f[2]
  return keys, pdfs


def verdicts(root):
  out = {}
  path = os.path.join(root, 'sweep_verdicts.tsv')
  if os.path.exists(path):
    for line in open(path):
      f = line.rstrip('\n').split('\t')
      out[(f[0], f[1])] = dict(status=int(f[2]), err=int(f[4]), fatal=int(f[5]), warn=int(f[6]),
                               secs=float(f[7]))
  return out


def validity(root):
  out = {}
  path = os.path.join(root, 'validate_verdicts.tsv')
  if os.path.exists(path):
    for line in open(path):
      f = line.rstrip('\n').split('\t')
      out[(f[0], f[1])] = int(f[2]) == 0
  return out


def xml_facts(path):
  try:
    raw = open(path, 'rb').read()
  except OSError:
    return None
  # A Rust primitive's `\meaning` prints its heap address (cnltx/cnltx_en): not a change.
  raw = re.sub(rb'(CODE\(|addr: |DynMetadata\()0x[0-9a-f]+', rb'\g<1>0x', raw)
  text = raw.decode('utf-8', 'replace')
  words = len(re.findall(r'\w+', re.sub(r'<[^>]*>', ' ', text)))
  tex = hashlib.sha1('\n'.join(sorted(re.findall(r'<Math [^>]*?tex="([^"]*)"', text)))
                     .encode()).hexdigest()[:12]
  return dict(sha=hashlib.sha1(raw).hexdigest(), words=words, tex=tex)


def recall(xml, pdf):
  r = subprocess.run(['python3', os.path.join(HERE, 'pdf_recall.py'), xml, pdf, '--min-len', '4',
                      '--show', '0'], capture_output=True, text=True)
  m = re.search(r'recall=([\d.]+)%', r.stdout)
  return float(m.group(1)) if m else None


def repro_rows(root):
  """{topic/name: failing?} from repros.sh's tables: `status expect rust name -- …`."""
  rows = {}
  for table in glob.glob(os.path.join(root, '*.txt')):
    topic = os.path.basename(table)[:-4]
    for line in open(table, errors='replace'):
      f = line.split()
      if len(f) < 4 or f[0] == 'status':
        continue
      status, expect, rust, name = f[0], f[1], f[2], f[3]
      if rust == 'noxml':
        failing = True
      elif not rust.isdigit():
        continue
      else:
        failing = (status == 'GREEN' and expect.isdigit() and int(rust) > int(expect)) or \
                  (status == 'CONTROL' and int(rust) == 0)
      rows[f'{topic}/{name}'] = (failing, rust)
  return rows


def main():
  out = sys.argv[1]
  with_recall = '--recall' in sys.argv[2:]
  keys, pdfs = net_keys()
  va, vb = verdicts(os.path.join(out, 'A')), verdicts(os.path.join(out, 'B'))
  ga, gb = validity(os.path.join(out, 'A')), validity(os.path.join(out, 'B'))
  tally = collections.Counter()
  changed, regressions = [], []
  for key in keys:
    bundle, name = key
    tally['manuals'] += 1
    if key not in va or key not in vb:
      missing = ' and '.join(s for s, v in (('A', va), ('B', vb)) if key not in v)
      tally['missing'] += 1
      regressions.append(f'{bundle}/{name}\tno verdict on {missing}')
      continue
    a, b = va[key], vb[key]
    xa = xml_facts(os.path.join(out, 'A', bundle, name, name + '.xml'))
    xb = xml_facts(os.path.join(out, 'B', bundle, name, name + '.xml'))
    tally['secs A'] += a['secs']
    tally['secs B'] += b['secs']
    notes, bad = [], False
    for field in ('err', 'fatal', 'warn'):
      if a[field] != b[field]:
        notes.append(f"{field} {a[field]}->{b[field]}")
        tally[f"{field}{'+' if b[field] > a[field] else '-'}"] += 1
    if a['status'] != b['status']:
      notes.append(f"status {a['status']}->{b['status']}")
      tally['status worse' if b['status'] > a['status'] else 'status better'] += 1
    bad |= b['status'] > a['status'] or b['fatal'] > a['fatal'] or b['err'] > a['err']
    if ga.get(key) and gb.get(key) is False:
      notes.append('schema-valid -> invalid')
      tally['valid-'] += 1
      bad = True
    elif ga.get(key) is False and gb.get(key):
      notes.append('schema-invalid -> valid')
      tally['valid+'] += 1
    if b['secs'] > 1.5 * a['secs'] and b['secs'] - a['secs'] > 5:
      notes.append(f"slower {a['secs']:.0f}->{b['secs']:.0f} s")
      tally['slower >50% and >5s'] += 1
    if xa and xb and xa['sha'] != xb['sha']:
      tally['xml differs'] += 1
      notes.append(f"xml differs, words {xa['words']}->{xb['words']}")
      if xb['words'] < 0.99 * xa['words']:
        tally['words-1%'] += 1
        bad = True
      elif xb['words'] > 1.01 * xa['words']:
        tally['words+1%'] += 1
      if xa['tex'] != xb['tex']:
        tally['tex= changed'] += 1
        notes.append('tex= changed')
        bad = True
      if with_recall and key in pdfs:
        ra = recall(os.path.join(out, 'A', bundle, name, name + '.xml'), pdfs[key])
        rb = recall(os.path.join(out, 'B', bundle, name, name + '.xml'), pdfs[key])
        notes.append(f"recall {ra}->{rb}")
        if ra is not None and (rb is None or rb < ra):
          tally['recall down'] += 1
          bad = True
    elif (xa is None) != (xb is None):
      notes.append('xml ' + ('lost' if xb is None else 'gained'))
      bad |= xb is None
    if notes:
      changed.append(f"{bundle}/{name}\t" + '; '.join(notes))
      if bad:
        regressions.append(changed[-1])
  # The repro catalog: complete on both sides, compared row by row.
  topics = sorted(os.path.basename(p.rstrip('/')) for p in glob.glob(os.path.join(HERE, 'repros', '*/')))
  for side in ('A', 'B'):
    status = os.path.join(out, f'repros.{side}', 'status.tsv')
    ran = [line.split('\t')[0] for line in open(status)] if os.path.exists(status) else []
    if sorted(ran) != topics:
      regressions.append(f'repros.{side}\tincomplete: {len(ran)} of {len(topics)} topics ran')
  ra_rows = repro_rows(os.path.join(out, 'repros.A'))
  rb_rows = repro_rows(os.path.join(out, 'repros.B'))
  tally['repros'] = len(rb_rows)
  already = sorted(k for k, (f, _) in rb_rows.items() if f and ra_rows.get(k, (False,))[0])
  for key in sorted(set(ra_rows) | set(rb_rows)):
    fa, rust_a = ra_rows.get(key, (None, '-'))
    fb, rust_b = rb_rows.get(key, (None, '-'))
    if fb and not fa:
      tally['repros newly failing'] += 1
      regressions.append(f'repros/{key}\tnewly failing ({rust_a} -> {rust_b})')
    elif rust_a != rust_b:
      changed.append(f'repros/{key}\t{rust_a} -> {rust_b}')
  for k in ('manuals', 'missing', 'status better', 'status worse', 'err+', 'err-', 'fatal+',
            'fatal-', 'warn+', 'warn-', 'valid+', 'valid-', 'xml differs', 'words-1%', 'words+1%',
            'tex= changed', 'recall down', 'slower >50% and >5s', 'repros', 'repros newly failing'):
    print(f"{k}: {tally[k]}")
  print(f"secs: A={tally['secs A']:.0f} B={tally['secs B']:.0f}")
  print(f"repros failing on both sides: {len(already)} {' '.join(already)}")
  print('\nCHANGED')
  print('\n'.join(changed) or '(none)')
  print('\nREGRESSIONS')
  print('\n'.join(regressions) or '(none)')


if __name__ == '__main__':
  main()
