//! TeX Job
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
use chrono::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Job Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // The current Job
  //----------------------------------------------------------------------
  // \jobname          c  is the underlying file name for a job.
  // \time             pi holds the current time in minutes after midnight (0-1439).
  // \day              pi holds the current day of the month (1-31).
  // \month            pi holds the current month of the year (1-12).
  // \year             pi holds the current year (e.g., 2000).
  // \mag              pi holds the magnification ratio times 1000.
  DefMacro!(T_CS!("\\jobname"), None, Tokens!()); // Set to the filename by initialization
  DefRegister!("\\time", Number!(0));
  DefRegister!("\\day", Number!(0));
  DefRegister!("\\month", Number!(0));
  DefRegister!("\\year", Number!(0));
  DefRegister!("\\mag", Number!(1000));

  // TODO: This may mess up Daemon state? Reinit when setting jobname?
  let dt = Local::now();
  AssignValue!("\\day", Number!(dt.day()), Scope::Global);
  AssignValue!("\\month", Number!(dt.month()), Scope::Global);
  AssignValue!("\\year", Number!(dt.year()), Scope::Global);
  AssignValue!(
    "\\time",
    Number!(60 * dt.hour() + dt.minute()),
    Scope::Global
  );

  //======================================================================
  // Random Job related things
  //----------------------------------------------------------------------
  // \end              c  terminates the current job.
  // \everyjob         pt holds tokens which are inserted at the start of every job.
  // \deadcycles       iq is the number of times \output was called since the last \shipout.
  // \maxdeadcycles    pi is the maximum allowed value of \deadcycles before an error is generated.
  DefPrimitive!("\\end", {
    gullet::flush();
  });
  DefRegister!("\\everyjob", Tokens!());
  DefRegister!("\\deadcycles", Number!(0));
  DefRegister!("\\maxdeadcycles", Number!(0));

  //======================================================================
  // Dumping
  //----------------------------------------------------------------------
  // \dump             c  outputs a format file in INITEX; otherwise it is equivalent to \end.

  DefMacro!("\\dump", {
    Warn!("unexpected", "dump", "Do not know how to \\dump yet, sorry");
  });

  // TODO: load_dump
  // TODO: load_latex
});
