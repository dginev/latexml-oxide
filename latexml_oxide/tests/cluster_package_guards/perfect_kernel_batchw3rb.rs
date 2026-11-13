//! W3RB (PR #829 Rhai round B): values cross the Rhai boundary with their types, as Perl's do. A value stored
//! from a script keeps its type and reads back as itself (Package.pm LookupValue/AssignValue :257/:261,
//! LookupMapping/AssignMapping :283/:287); Digest takes a string (TokenizeInternal'd, :393) and AssignCatcode a
//! scope (:299); a macro closure's Token/Tokens is its expansion; the Read* gullet calls return Tokens; the
//! code-table calls (:338-381) and LookupMappingKeys (:291) exist. Each row is pinned against Perl `latexml`
//! with the equivalent `.ltxml` binding (Perl 0 Errors, 0 Warnings).
#![cfg(feature = "runtime-bindings")]

use latexml::util::test::{assert_element, convert_files_with};

use crate::perfect_kernel_batch46::{error_count, warning_count};

const LXRB: &str = r#"AssignBool("lxrb:f", false);
AssignNumber("lxrb:n", 7);
AssignValue("lxrb:t", Tokenize("\\textbf{T}"));
AssignMapping("lxrb:m", "k", T_CS("\\relax"));
AssignMapping("lxrb:m", "j", true);
DefMacro("\\vtypes", || [type_of(LookupValue("lxrb:f")), type_of(LookupValue("lxrb:n")),
  type_of(LookupValue("lxrb:t")), type_of(LookupMapping("lxrb:m", "k")),
  type_of(LookupMapping("lxrb:m", "j"))].reduce(|s, x| if s == () { x } else { s + "," + x }));
DefMacro("\\vtok", || LookupValue("lxrb:t"));
DefMacro("\\vif", || if LookupValue("lxrb:f") { "Y" } else { "N" });
DefMacro("\\mkeys", || { let s = ""; for k in LookupMappingKeys("lxrb:m") { s += k; } s });
AssignValue("lxrb:dg", ToString(Digest("D\\relax g")));
AssignValue("lxrb:dt", ToString(DigestText(Tokenize("Dt"))));
DefMacro("\\dg", || LookupValue("lxrb:dg") + LookupValue("lxrb:dt"));
DefPrimitive("\\catg", || AssignCatcode("Q", 12, "global"));
DefPrimitive("\\catl", || AssignCatcode("R", 12, "local"));
DefMacro("\\cattype", || if LookupCatcode("☃") == () { "undef" } else { "def" });
DefMacro("\\retT", || T_CS("\\textbf"));
DefMacro("\\retTs", || Tokenize("\\emph{E}"));
DefMacro("\\rdx", || ReadArg());
DefMacro("\\rdu", || ReadUntil("."));
DefMacro("\\rdo", || { let o = ReadOptional(); if o == () { "none" } else { o } });
DefMacro("\\codes", || { AssignSFcode("Q", 1001); AssignMathcode("Q", 0x7151, "global"); AssignLCcode("Q", 0x72); AssignUCcode("q", 0x52); AssignDelcode("Q", 0x123);
  [LookupMathcode("a"), LookupLCcode("A"), LookupUCcode("a"), LookupSFcode("A"),
   LookupSFcode("Q"), LookupMathcode("Q"), LookupLCcode("Q"), LookupUCcode("q"), LookupDelcode("Q"),
   if LookupLCcode("\u2603") == () { "undef" } else { "def" }].reduce(|s, x| if s == () { `${x}` } else { s + "," + x }) });
"#;

const LXRB_TEX: &str = r"\documentclass{article}
\usepackage{lxrb}
\begin{document}
\vtypes

\vtok \vif \mkeys \dg

{\catg}\the\catcode`\Q{} {\catl}\the\catcode`\R{} \cattype

\retT{X} \retTs

\catcode`\!=11 \def\a!b{AB}\rdx{\a!b}\catcode`\!=12 \rdu x. \rdo[o] \rdo

\codes
\end{document}
";

/// Rows, each red on e2cbae128e3, with Perl's output:
/// p1 (ours only; Perl has no type names): a bool, a number, Tokens and a Token read back as themselves (was
/// `string`, and `AssignValue`/`AssignMapping` of a token value was "Function not found").
/// p2: `\vtok` expands the stored Tokens (bold T); `if LookupValue(false)` is false (N); `LookupMappingKeys` sorted
/// (`jk`, Perl's sort); `Digest` of a string and `DigestText` of Tokens (`DgDt`).
/// p3: `AssignCatcode` with a scope (global 12, local reverted to 11); `LookupCatcode` of an unset char is `()`
/// (Perl undef; was 12).
/// p4: a macro closure returning a Token / Tokens expands to it (was the type name `Token`/`Tokens` as text).
/// p5: `ReadArg`/`ReadUntil`/`ReadOptional` return Tokens: `\a!b` read under catcode 11 stays one control sequence
/// (was re-tokenized as `\a` `!b`, an undefined-macro Error); an absent optional is `()` (Perl undef).
/// p6: the Mathcode/Delcode/LCcode/UCcode/SFcode lookups and assigns (was "Function not found").
#[test]
fn rhai_values_keep_their_types() {
  let (log, xml) = convert_files_with(LXRB_TEX, &[("lxrb.sty.rhai", LXRB)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for (id, para) in [
    (
      "p1",
      r#"<para xml:id="p1"><p>bool,i64,Tokens,Token,bool</p></para>"#,
    ),
    (
      "p2",
      r#"<para xml:id="p2"><p><text font="bold">T</text>NjkDgDt</p></para>"#,
    ),
    ("p3", r#"<para xml:id="p3"><p>12 11 undef</p></para>"#),
    (
      "p4",
      r#"<para xml:id="p4"><p><text font="bold">X</text> <emph font="italic">E</emph></p></para>"#,
    ),
    ("p5", r#"<para xml:id="p5"><p>ABx o none</p></para>"#),
    (
      "p6",
      r#"<para xml:id="p6"><p>29025,97,65,999,1001,29009,114,82,291,undef</p></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
}
