#!/usr/bin/env python3
"""Guard-strength upgrade pipeline (PLANS.md item 5, batch B1).

Rewrites bare-tag assertions of the shape

    assert!(
      x.contains("<TAG"),
      "message:\n{x}"
    );

in one integration-test file into whole-element snapshots

    latexml::util::test::assert_element(&x, "TAG", &[], r##"<TAG …>…</TAG>"##);

by (1) rewriting each assertion with a PENDING expectation, (2) running each
affected test under nextest and harvesting the element that
`assert_element` prints in its failure message, (3) filling the expectation
in. Re-run the test binary afterwards: every upgraded test must be green, and
the pinned element must be read once by a human — a snapshot pins what the
engine does today, which is only a guard if that output is right.

    tools/perfect_kernel/test_snapshot.py <tests/file.rs> <test-binary-name> <tag>

Requires the prebuilt test binary (CARGO_TARGET_DIR honoured) and the vendor
TeX Live first on PATH. `assert_element` normalizes indentation and attribute
order (`latexml_oxide/src/util/test.rs`).
"""
import re, subprocess, sys, os
path, testbin, tag = sys.argv[1], sys.argv[2], sys.argv[3]
src = open(path).read()
# Phase 1: rewrite bare-tag assertions to assert_element with a pending expectation.
pat = re.compile(r'''  assert!\(\n    (\w+)\.contains\("<%s"\)(?: \|\| \1\.contains\("%s"\))?(?: && \1\.contains\("([^"\n]*)"\))?,\n    "((?:[^"\\]|\\.)*?):?\\n\{\1\}"\n  \);\n''' % (tag, tag))
def repl(m):
    var, text, msg = m.group(1), m.group(2), m.group(3)
    out = f'  // {msg}: the WHOLE first <{tag}> element is pinned.\n  latexml::util::test::assert_element(&{var}, "{tag}", &[], r##"PENDING"##);\n'
    if text:
        out += f'  assert!({var}.contains("{text}"), "{msg}:\\n{{{var}}}");\n'
    return out
new, n = pat.subn(repl, src)
print("rewritten assertions:", n)
open(path, 'w').write(new)
# which tests contain PENDING?
tests = []
for m in re.finditer(r'^([ \t]*)fn (\w+)\(\) \{(.*?)\n\1\}\n', new, re.S | re.M):
    if 'r##"PENDING"##' in m.group(3):
        tests.append(m.group(2))
print("tests:", len(tests))
# Phase 2: run each test, harvest the found element.
found = {}
for t in tests:
    out = subprocess.run(['taskset','-c','64-127','cargo','nextest','run','--build-jobs','48','-j','48','-p','latexml','--test',testbin,'-E',f'test(/(^|::){t}$/)','--no-fail-fast'], capture_output=True, text=True); out = out.stdout + out.stderr
    m = re.search(r'--- found:\n(.*?)\n *--- expected:\n', out, re.S)
    if m:
        found[t] = '\n'.join(l[4:] if l.startswith('    ') else l for l in m.group(1).split('\n'))
    else:
        print("NO ELEMENT FOR", t, "(no <%s> in output, or another assertion failed first)" % tag)
        m2 = re.search(r'panicked at [^\n]*\n(.*?)\n', out, re.S)
        if m2: print("   ", m2.group(1)[:200])
# Phase 3: fill the expectations, in order of appearance per test.
def fill(m):
    indent, name, body = m.group(1), m.group(2), m.group(3)
    if name in found:
        el = found[name].replace('"##', '"# #')
        body = body.replace('r##"PENDING"##', 'r##"' + el + '"##', 1)
    return f'{indent}fn {name}() {{{body}\n{indent}}}\n'
final = re.sub(r'^([ \t]*)fn (\w+)\(\) \{(.*?)\n\1\}\n', fill, new, flags=re.S | re.M)
open(path, 'w').write(final)
print("filled:", len(found), "of", len(tests))
