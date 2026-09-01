use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // The real assoccnt.sty pulls in xcolor/etoolbox/xkeyval/xstring
  // (assoccnt.sty L21-24); packages raw-loaded ON TOP of this binding
  // (cntperchap.sty) rely on that transitive surface (\define@boolkey,
  // \presetkeys, \DeclareOptionX from xkeyval — witness cntperchap docs).
  RequirePackage!("xcolor");
  RequirePackage!("etoolbox");
  RequirePackage!("xkeyval");
  RequirePackage!("xstring");
  // assoccnt.sty (Christian Hupfer) — "associated counters" stepped alongside
  // driver counters. The raw package WRAPS the kernel counter commands
  // (\stepcounter/\addtocounter/\refstepcounter/\setcounter, assoccnt.sty
  // L470-571) with bookkeeping that routes every counter operation through
  // \setkeys on its own key family. Under our engine those wrappers also fire
  // inside CONSTRUCTION-time counter stepping (listings line numbers), and
  // their raw expansion leaked #PCDATA/<ltx:text> into <ltx:listing> — the
  // assoccnt manual's 385-error cascade (perfect-kernel sweep 13). The
  // association bookkeeping has no bearing on XML content, so bind the
  // declaration surface as noops and leave the kernel counter commands
  // UNWRAPPED (real listings steps \c@lstnumber at the internals level
  // anyway, so associated counters never see line stepping in real TeX
  // either).
  def_macro_noop("\\AddDriverCounter[]{}")?;
  def_macro_noop("\\ClearDriverCountersList")?;
  DefMacro!("\\IsDriverCounter{}{}{}", "#3");
  def_macro_noop("\\AddAssociatedCounters[]{}{}")?;
  def_macro_noop("\\DeclareAssociatedCounters[]{}{}")?;
  def_macro_noop("\\RemoveAssociatedCounter{}{}")?;
  def_macro_noop("\\RemoveAssociatedCounters{}{}")?;
  def_macro_noop("\\ClearAssociatedCountersList{}")?;
  DefMacro!("\\IsAssociatedToCounter{}{}{}{}", "#4");
  DefMacro!("\\IsAssociatedCounter{}{}{}", "#3");
  DefMacro!("\\IsInResetList{}{}{}{}", "#4");
  DefMacro!("\\IsSuspendedCounter{}{}{}", "#3");
  def_macro_noop("\\SuspendCounters[]{}")?;
  def_macro_noop("\\ResumeSuspendedCounters")?;
  def_macro_noop("\\AssociatedDriverCounterInfo{}")?;
  def_macro_noop("\\AssociationStatistics[]")?;
  def_macro_noop("\\PrettyPrintCounterName[]{}")?;
  DefMacro!("\\GetDriverCounter{}", "#1");
  def_macro_noop("\\LastSteppedCounter")?;
  def_macro_noop("\\LastSetCounter")?;
  def_macro_noop("\\LastRefSteppedCounter")?;
  def_macro_noop("\\LastAddedToCounter")?;
});
