import re, subprocess, sys, os
bibfile, target = sys.argv[1], sys.argv[2]  # full bib path, bib name used by min.tex
text = open(bibfile, encoding='utf-8', errors='replace').read()
idxs = [m.start() for m in re.finditer(r'(?m)^\s*@', text)]
idxs.append(len(text))
head = text[:idxs[0]]
ents = [text[idxs[i]:idxs[i+1]] for i in range(len(idxs)-1)]
# keep @string/@preamble always
keep_always = [e for e in ents if re.match(r'\s*@\s*(string|preamble|comment)', e, re.I)]
cand = [e for e in ents if e not in keep_always]
def fails(subset):
    open(target, 'w', encoding='utf-8').write(head + "".join(keep_always) + "".join(subset))
    r = subprocess.run(['/home/deyan/git/latexml-oxide/target/release/latexml_oxide','min.tex','--dest=outmin/min.html'],
                       capture_output=True, text=True, timeout=300)
    log = re.sub(r'\x1b\[[0-9;]*m','', r.stderr + r.stdout)
    return any(s in log for s in ('close a group that switched to mode math','bibliography:convert','Attempt to end mode restricted_horizontal','Excessive recursion'))
assert fails(cand), "full set does not fail?"
while len(cand) > 1:
    half = len(cand)//2
    a, b = cand[:half], cand[half:]
    if fails(a): cand = a
    elif fails(b): cand = b
    else:
        # culprit needs both halves? try minimizing pairwise — fall back: shrink a by moving pivot
        print("NEITHER HALF FAILS ALONE — interaction; stopping with", len(cand)); break
print("=== culprit entry (first 40 lines):")
print("\n".join(cand[0].splitlines()[:40]))
