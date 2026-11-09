//! T1, the tests at `LATEXML_FM_EMIT=2` before the flip (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980): the one per-switch expectation a guard keeps where the emitter's output is as right as
//! the switch-off one, so the same guard passes under `LATEXML_FM_EMIT=0` and at `=2`, the default since the flip (FLIP1).
//! When the old driver goes (Z18-Z19), each `expect(off, two)` becomes its `two`.

/// The frontmatter emitter's switch as a conversion of this test process reads it: `LATEXML_FM_EMIT`, parsed as the engine's
/// `fm_emit_mode` parses it (2, the default, when unset or unreadable). A guard using [`expect`] never sets the engine's per-thread override
/// (`set_fm_emit_override`), so the environment is the whole switch.
pub(super) fn fm_emit() -> u8 {
  std::env::var("LATEXML_FM_EMIT")
    .ok()
    .and_then(|v| v.trim().parse().ok())
    .unwrap_or(2)
}

/// A guard's expectation under the switch its conversion runs with: `two` at `LATEXML_FM_EMIT=2`, `off` otherwise (the switch
/// off, `0`, and 1, the reader still running at the call).
pub(super) fn expect<T>(off: T, two: T) -> T { if fm_emit() == 2 { two } else { off } }

/// A guard whose switch-off expectation is right but whose `LATEXML_FM_EMIT=2` output is a known wrong reading — a named RED,
/// fixed after the flip (user 2026-11-08: guards for shapes with no measured reach stay named REDs failing only at `=2`): at
/// `=2` it prints the RED and returns true, so the guard skips its assertions there and keeps them under `LATEXML_FM_EMIT=0` (run
/// once per bundle until the old driver's deletion); that bundle turns each such guard into an `#[ignore = "RED at =2: …"]` guard
/// keeping the right expectation.
pub(super) fn red_at_two(red: &str) -> bool {
  let two = fm_emit() == 2;
  if two {
    eprintln!("RED at LATEXML_FM_EMIT=2: {red}");
  }
  two
}

/// [`red_at_two`]'s twin for a guard whose switch-off output is the known wrong reading (a user ruling the old driver
/// predates): under `LATEXML_FM_EMIT=0` (and 1) it prints the RED and returns true, so the guard keeps its right expectation
/// for the switch that is fixed to it.
pub(super) fn red_off(red: &str) -> bool {
  let off = fm_emit() != 2;
  if off {
    eprintln!("RED at LATEXML_FM_EMIT={}: {red}", fm_emit());
  }
  off
}
