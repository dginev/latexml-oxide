use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("psfig");
  RequirePackage!("epsf");
  RequirePackage!("epsfig");
  // Perl-parity stubs: every macro below mirrors
  // ar5iv-bindings/ws-p8-50x6-00.cls.ltxml L22-47 one-for-one. "INCOMPLETE"
  // label was inaccurate — Perl's own support is identical journal-abbrev
  // forwarding, not a richer port.
  DefMacro!("\\Journal{}{}{}{}", "{#1} {\\bf #2}, #3 (#4)");
  DefMacro!("\\NCA", "\\em Nuovo Cimento");
  DefMacro!("\\NIM", "\\em Nucl. Instrum. Methods");
  DefMacro!("\\NIMA", "{\\em Nucl. Instrum. Methods} A");
  DefMacro!("\\NPB", "{\\em Nucl. Phys.} B");
  DefMacro!("\\PLB", "{\\em Phys. Lett.}  B");
  DefMacro!("\\PRL", "\\em Phys. Rev. Lett.");
  DefMacro!("\\PRD", "{\\em Phys. Rev.} D");
  DefMacro!("\\ZPC", "{\\em Z. Phys.} C");
  DefMacro!("\\st", "\\scriptstyle");
  DefMacro!("\\sst", "\\scriptscriptstyle");
  DefMacro!("\\mco", "\\multicolumn");
  DefMacro!("\\epp", "\\epsilon^{\\prime}");
  DefMacro!("\\vep", "\\varepsilon");
  DefMacro!("\\ra", "\\rightarrow");
  DefMacro!("\\ppg", "\\pi^+\\pi^-\\gamma");
  DefMacro!("\\vp", "{\\bf p}");
  DefMacro!("\\ko", "K^0");
  DefMacro!("\\kb", "\\bar{K^0}");
  DefMacro!("\\al", "\\alpha");
  DefMacro!("\\ab", "\\bar{\\alpha}");
  DefMacro!("\\be", "\\begin{equation}");
  DefMacro!("\\ee", "\\end{equation}");
  DefMacro!("\\bea", "\\begin{eqnarray}");
  DefMacro!("\\eea", "\\end{eqnarray}");
  DefMacro!("\\CPbar", "\\hbox{{\\rm CP}\\hskip-1.80em{/}}");
});
