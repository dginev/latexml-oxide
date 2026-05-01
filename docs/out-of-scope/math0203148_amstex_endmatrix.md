# math0203148 — `\matrix...\endmatrix` mode-stack mismatch in amsppt

**Status:** AmS-TeX/amsppt long-tail. P=0 R=2 (Cluster H-adjacent),
deferred 2026-05-01.

## Pattern

```latex
\input amstex
\documentstyle{amsppt}
...
$$\matrix
A & B & C \\
\downarrow && \downarrow \\
D & E & F \\
\endmatrix$$
```

Rust emits 2 errors at the `\endmatrix` lines (3088 and 3108):

```
Error:unexpected:\lx@end@gen@matrix Attempt to close a group that switched to mode display_math; current frame is mode-switch to display_math due to T_CS[\lx@begin@display@math]
```

Perl produces 0 errors on the same paper.

## Triage notes

* `\lx@end@gen@matrix` is bound identically in Perl
  (`Base_XMath.pool.ltxml:600`: `DefPrimitive('\lx@end@gen@matrix',
  sub { $_[0]->egroup; })`) and Rust (`base_xmath.rs:901`).
* `\matrix`/`\endmatrix` AmS-TeX bindings: `amsmath_sty.rs:285`
  binds `\endmatrix` → `\lx@end@ams@matrix` (NOT `gen@matrix`),
  but the error mentions `gen@matrix`. So `\matrix` in this file
  is reaching the `gen@matrix` path — likely via amsppt-specific
  routing not yet ported. Need to track the binding chain:
  amsppt's `\matrix` → ?...
* Two consecutive `$$\matrix...\endmatrix$$` blocks both error.
  The errors fire AT the `\endmatrix` line — meaning the matrix
  group/mode is already in an inconsistent state when end runs.

## Why deferred

This is part of the broader AmS-TeX long-tail
(see `math0606553_xy_compile.md`, `math0005251_math_parser_oom.md`,
etc.). The `\input amstex \documentstyle{amsppt}` papers each
expose distinct AmS-TeX semantics that need careful porting.

The `\matrix`/`\endmatrix` binding in amsppt context appears to
mismatch the `gen@matrix` group balancing — fix needs reading
the full amsppt `\matrix` body and ensuring it opens with
`\lx@begin@gen@matrix` (which calls `bgroup`) when later closed
by `\endmatrix` via `\lx@end@gen@matrix` (egroup).

## Possible fix locus

Probably `amsmath_sty.rs` or `amsppt_sty.rs`: bind `\matrix`
via the `\lx@gen@plain@matrix{...}{...}` path so the begin/end
group pair is consistent. Alternatively `\endmatrix` should
route to `\lx@end@gen@matrix` rather than `\lx@end@ams@matrix`
for amsppt mode (or both should pop the same group).
