//! turing.sty — Turing machine simulation
//! Perl: turing.sty.ltxml — 222 lines
//! Simulates Turing machines with tape, states, and transition rules.
//! Niche package — provides stub definitions for the user-facing macros.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Tape symbols — Perl L25-26
  DefMacro!("\\newworld", "=");
  DefMacro!("\\blank", "-");

  // Display macros — Perl L33-60
  DefConstructor!("\\spec{}", "<ltx:text cssstyle='text-decoration:underline'>#1</ltx:text>",
    enter_horizontal => true);
  DefConstructor!("\\speca{}", "<ltx:text cssstyle='border:1px solid'>#1</ltx:text>",
    enter_horizontal => true);

  // Machine control — Perl L70-222
  // These require complex TeX macro expansion for Turing machine simulation.
  // Stubbed: the machine doesn't actually execute, but documents can load.
  DefMacro!("\\newtm{}", "");
  DefMacro!("\\showtm", "");
  DefMacro!("\\runtm", "");
  DefMacro!("\\stepandshow{}", "");
  DefMacro!("\\loopstep{}", "");
  DefMacro!("\\nextstep", "");
  DefMacro!("\\findstate", "");
  DefMacro!("\\findrule", "");
  DefMacro!("\\findr{}{}{}", "");
});
