// Daemon-suite fixture test redone with local `.rhai` bindings — a faithful port
// of Perl `t/daemon/testlocks*` (never previously wired into the Rust suite).
//
// `testlocks` exercises `locked` macro semantics across package-load order: a
// locked macro CAN be re-defined by another binding load (loads run UNLOCKED),
// but NOT by raw TeX (`\def` in testlocks-a.sty) or document source. The Perl
// test is driven in daemon mode with `preload = testlocks.sty`; the Rust harness
// has no per-test preload, so `testlocks.tex` `\usepackage{testlocks}` instead
// (the only observable difference is the `class`/`package` processing-
// instruction order — the body, i.e. every `\fooA..\fooE` outcome, matches the
// Perl fixture exactly).
//
// Resolution: `\usepackage{testlocks}` → `testlocks.sty.rhai`; that
// `InputDefinitions("testlocks-a", noltxml)` raw-loads `testlocks-a.sty`, whose
// `\RequirePackage{testlocks-b}` → `testlocks-b.sty.rhai`. All via the shared
// binding-resolution chain over the source-dir search path, so this is gated to
// `runtime-bindings`.
#![cfg(feature = "runtime-bindings")]

use latexml::tex_tests;

tex_tests!("tests/daemon_rhai");
