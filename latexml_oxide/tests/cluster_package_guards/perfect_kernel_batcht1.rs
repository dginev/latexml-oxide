//! T1, the tests at `LATEXML_FM_EMIT=2` before the flip (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980): the one per-switch expectation a guard keeps where the emitter's output is as right as
//! the switch-off one, so the same guard passes under the default and under `LATEXML_FM_EMIT=2`. At the flip each
//! `expect(off, two)` becomes its `two`.

/// The frontmatter emitter's switch as a conversion of this test process reads it: `LATEXML_FM_EMIT`, parsed as the engine's
/// `fm_emit_mode` parses it (0 when unset or unreadable). A guard using [`expect`] never sets the engine's per-thread override
/// (`set_fm_emit_override`), so the environment is the whole switch.
pub(super) fn fm_emit() -> u8 {
  std::env::var("LATEXML_FM_EMIT")
    .ok()
    .and_then(|v| v.trim().parse().ok())
    .unwrap_or(0)
}

/// A guard's expectation under the switch its conversion runs with: `two` at `LATEXML_FM_EMIT=2`, `off` otherwise (the switch
/// off, the default, and 1, the reader still running at the call).
pub(super) fn expect<T>(off: T, two: T) -> T { if fm_emit() == 2 { two } else { off } }
