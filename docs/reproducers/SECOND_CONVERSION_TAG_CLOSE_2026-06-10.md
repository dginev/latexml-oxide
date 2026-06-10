# Second-conversion-in-process: spurious malformed-close on tags-in-text

**Symptom** (log-only; XML byte-identical, assertions green):
`Error:malformed:ltx:text Attempt to close </ltx:text>, which isn't open.
Currently in #document<ltx:document><ltx:para><ltx:p>` in
`30_script_bindings` when BOTH tests run (any order/threading), at
document.rs close_element (~L1068).

## Established by bisection + backtrace (2026-06-10)

1. Fires in the SECOND conversion of a process; serial AND parallel —
   so the poisoned state is PROCESS-GLOBAL (thread-locals incl. the
   `#[thread_local] MODEL` are fresh per test thread in parallel runs).
   On this codebase process-global ⇒ libxml2 (init/dictionaries/etc.)
   or something reached through the Document veneer. (User hypothesis,
   corroborated.)
2. First conversion need NOT involve runtime bindings (probe: plain
   native article doc as conversion 1 still triggers it).
3. Failing construct in conversion 2: the `\numbered` runtime-template
   specimen `<ltx:text class="eq">#tags #1</ltx:text>` — backtrace:
   wire.rs template_replacement → apply_ops → exec_ops → close_element;
   the close fails right after absorbing three `\lx@tag@intags`
   whatsits (RefStepCounter properties), i.e. absorbing the tag
   whatsits left current-node tracking at <p> although the final tree
   is identical to the clean run.
4. `\numbered{NUM}` ALONE in conversion 2 does NOT reproduce — needs
   preceding paragraph content (full e2e doc does).
5. Output-neutral: clean vs contaminated XML diff is empty.

## Next probes

- Temp-instrument the auto-close/insertion decision for inserting
  `ltx:tag` while `ltx:text` is open (document.rs find-insertion /
  model can_contain path): log the decision in conversion 1 vs 2.
- Audit Document Drop / reset_thread_engine / any libxml cleanup-init
  pairs (xmlCleanupParser mid-process would be a bug), and identity-
  sensitive node comparisons in close_element's ancestor walk
  (cf. WISDOM #58 pointer-identity precedent).
- When fixed: assert zero `Error:` lines in 30_script_bindings (serial
  + parallel) to pin the class; check the --server multi-conversion
  path which has the same exposure with purely native bindings.

Repro: `cargo test -p latexml --test 30_script_bindings -- --test-threads=1`
(both tests; grep log for `malformed:ltx:text`).
