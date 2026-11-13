---
name: latexml-surpass-perl
description: Evaluate and document a proposed intentional latexml-oxide improvement where Perl LaTeXML also fails or produces inferior behavior. Use only for a proposed new beyond-Perl semantic divergence, not for an ordinary Rust-only parity bug or an already documented divergence.
---

# Decide whether Rust should intentionally surpass Perl

The conservative default is faithful parity. A new divergence becomes a lasting
contract and therefore requires explicit user approval, first-principles evidence,
tests, and documentation in the same change.

## Establish the premise

1. Prove on the same host that Rust and Perl share the failure. If Rust alone
   fails, use `latexml-perl-port`; this workflow does not excuse a translation
   bug.
2. Establish the intended TeX or LaTeX behavior from primary sources: actual
   `tex.web`, current kernel/package definitions, and `pdflatex` or the relevant
   TeX engine on a minimal witness. Do not infer the specification from a nicer
   output shape.
3. Search the Perl source, its comments/history where available, and the existing
   `docs/parity/OXIDIZED_DESIGN*` entries. The apparent defect may protect an edge
   case or already have an approved Rust policy.

## Qualification gates

A new divergence should pass all of these:

- It corrects a general mechanism or compatibility defect, not a one-paper
  special case or cosmetic difference.
- The fix has a principled engine/package-level shape and preserves the intended
  TeX semantics.
- It does not silently change currently correct documents. Build a control set
  around neighboring inputs and measure any output change.
- Multiple independent witnesses support the class when corpus leverage is the
  justification. For a large claimed cluster, sample at least five before
  generalizing.
- The improvement would make sense for upstream Perl too, unless an existing
  documented Rust architecture necessarily differs.

If any gate is unsupported, keep the shared failure classified and record the
open evidence instead of landing a speculative binding.

## Approval package

Before implementation, present the user with:

- Perl, Rust, and real-TeX behavior on the minimal failing/control pair;
- the mechanism and proposed general fix;
- affected witnesses and neighboring regression risks;
- a draft numbered divergence entry;
- whether an upstream LaTeXML report is appropriate.

Stop for approval on a new divergence shape. Reusing an existing numbered policy
does not need new conceptual approval, but must cite and remain within that policy.

## Land atomically

After approval, include:

1. the implementation;
2. focused failing/control guards and representative witnesses;
3. a numbered entry in `docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md` describing
   Perl behavior, Rust behavior, primary-source rationale, witnesses, and upstream
   disposition;
4. a code comment referencing `OXIDIZED_DESIGN #N`;
5. an entry in `docs/parity/KNOWN_PERL_ERRORS.md` when the Perl behavior is a
   genuine upstream bug.

Run parity controls and the full applicable validation. Do not split code and its
contract documentation into separate landings.

