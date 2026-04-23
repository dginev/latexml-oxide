# Known Rust-side crashes reproducible with small/medium .tex inputs

## 0705.0790 — `latexmlpost_oxide` SIGSEGV (cycle 236, 2026-04-23)

**Paper:** hep-th supergravity, 315 KB .tex, ~5900 line `<document>` XML
after conversion.

**Perl:** succeeds, exit 0, ~91 s with 61 warnings.

**Rust `latexml_oxide`:** succeeds, exit 0, ~5 s, peak RSS ~1 GB.
Produces a 5.8 MB XML.

**Rust `latexmlpost_oxide`:** **reliably SIGSEGV** (5/5 runs) when fed
either the Rust-produced XML or the Perl-produced XML. Happens with
all tested output formats (`--format=html5`, `--format=html`,
`--format=xhtml`, and the default XML writer), with or without
`--dest`. The HTML output (~2 MB) reaches stdout before the crash
fires on teardown/cleanup — so the bug is in drop order of the
libxml2 document tree, not in the emission path itself.

**Why cortex_worker reports abort exit 134:** the benchmark harness
runs with a 60 s watchdog; the actual segfault in post-processing
escalates to SIGABRT via the watchdog thread. Changing the default
watchdog timeout does not help — the underlying post-proc crash is
deterministic.

**Reproducer steps:**
```sh
cd /tmp && mkdir -p repro && cd repro
cp ~/git/latexml-oxide/docs/known_crashes/0705_0790.tex .
cd ~/git/latexml-oxide
target/release/latexml_oxide \
    --preload=ar5iv.sty \
    --path=~/git/ar5iv-bindings/bindings \
    --dest=/tmp/repro/out.xml /tmp/repro/0705_0790.tex
# ^ succeeds in ~5s
target/release/latexmlpost_oxide --dest=/tmp/repro/out.html /tmp/repro/out.xml
# ^ SIGSEGV every time
```

**Next steps to investigate:**
1. Run under `valgrind --track-origins=yes` or Miri if build supports
   it — the teardown signature matches the documented libxml UAF
   family (WISDOM #36-37 + wisdom_lazy_xmlnode_ref.md + the
   replace_node / idstore / node_clone UAFs).
2. Binary-search the XML body to find which element subtree triggers
   the drop crash. The full 5900-line `<document>` is too big; try
   halving iteratively after wrapping in a valid `<document>` stub.
3. Check whether shrinking below the arena threshold (e.g. <1 MB
   XML) makes the crash disappear — would point to `finalize()` /
   idstore rebuild ordering.

**Status:** not a conversion regression. Core `latexml_oxide` converts
cleanly; only `latexmlpost_oxide` crashes. 1/512 failure rate on the
10k-sandbox benchmark slice corresponds to this paper (and ~19/7898 in
the full sandbox — some of those 19 may share this root cause).
