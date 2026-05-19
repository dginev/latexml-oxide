//! elsart_support.sty — Elsevier article support (non-core additions)
//! Perl: elsart_support.sty.ltxml — 175 lines
//! Loads elsart_support_core and adds theorem/proof/section formatting
use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("elsart_support_core");

  // Perl elsart_support.sty.ltxml L23-24:
  //   if (LookupValue('@amsthm')) { RequirePackage('amsthm'); }
  // Papers that pass the `amsthm` class option get amsthm's
  // \newtheorem{…} machinery; without it, elsart's theorem-like
  // environments fall through to the plain-stub defaults below.
  if state::lookup_bool_sym(pin!("@amsthm")) {
    RequirePackage!("amsthm");
  }

  // Theorem stubs (if amsthm not loaded)
  def_macro_noop("\\theoremstyle{}")?;
  // \qed and \ltx@qed now live in elsart_support_core_sty.rs (so plain
  // elsarticle papers get them without loading elsart_support).

  // Math symbols — Perl L37-42 (double-struck set notation)
  DefMath!("\\Cset", "\u{2102}", role => "ID", meaning => "complexes");
  DefMath!("\\Hset", "\u{210D}", role => "ID", meaning => "upper-complexes");
  DefMath!("\\Nset", "\u{2115}", role => "ID", meaning => "numbers");
  DefMath!("\\Qset", "\u{211A}", role => "ID", meaning => "rationals");
  DefMath!("\\Rset", "\u{211D}", role => "ID", meaning => "reals");
  DefMath!("\\Zset", "\u{2124}", role => "ID", meaning => "integers");

  // Fraction shortcuts — Perl L44-46
  DefMacro!("\\half", "{\\textstyle\\frac{1}{2}}");
  DefMacro!("\\threehalf", "{\\textstyle\\frac{3}{2}}");
  DefMacro!("\\quart", "{\\textstyle\\frac{1}{4}}");

  // Perl L48-49: differential and exponential unicode forms
  DefMath!("\\d", "\u{2146}", role => "DIFFOP", meaning => "differential-d");
  DefMath!("\\e", "\u{2147}", role => "ID", meaning => "exponential-e");

  // Perl L58: \pol (rightwards arrow overaccent)
  DefMath!("\\pol Digested", "\u{2192}", operator_role => "OVERACCENT");

  // Perl L51-53: elsart_support redefines \operatorname to ALWAYS emit
  //   <ltx:XMWrap role='OPERATOR'> — distinct from amsopn's OPFUNCTION-when-
  // unstarred / OPERATOR-when-starred split. Prior Rust port delegated to
  // amsopn unchanged (comment incorrectly claimed "both produce OPERATOR
  // markup"), so an elsart document's `\operatorname{lim}` silently got
  // role='OPFUNCTION' instead of 'OPERATOR'. Restore the override.
  DefConstructor!("\\operatorname OptionalMatch:* {}",
    "<ltx:XMWrap role='OPERATOR' scriptpos='#scriptpos'>#2</ltx:XMWrap>",
    bounded => true, require_math => true, font => { family => "serif" },
    properties => sub[args] {
      let scriptpos = if args[0].is_some() { "mid" } else { "post" };
      Ok(stored_map!("scriptpos" => scriptpos))
    });

  // Perl L55-56: \astsymbol{n}, \fnstar{n} — n-repeated Unicode char
  DefMacro!("\\astsymbol{}", sub[(n)] {
    let count = n.to_string().trim().parse::<usize>().unwrap_or(1);
    Ok(Tokens!(T_OTHER!("\u{2217}".repeat(count))))
  });
  DefMacro!("\\fnstar{}", sub[(n)] {
    let count = n.to_string().trim().parse::<usize>().unwrap_or(1);
    Ok(Tokens!(T_OTHER!("\u{22C6}".repeat(count))))
  });

  // Perl elsart_support.sty.ltxml does NOT define a `{proof}` environment.
  // Papers that use `\begin{proof}` either define it locally with
  // `\newenvironment{proof}` (e.g. 0801.1844) or pull it in via the
  // [amsthm] class option (which RequirePackages amsthm.sty.ltxml,
  // which Lets `\begin{proof}` to `\begin{@proof}`). Defining it here
  // unconditionally pre-empts user redefinitions and forces an
  // `<ltx:proof>` wrapper whose `<ltx:title>` puts the body's BOUND_MODE
  // into `restricted_horizontal`, breaking `$$...$$` shorthand which
  // requires BOUND_MODE to end with `vertical` (TeX_Math.pool L65).
  // Removing this restores Perl-faithful behavior.

  // Section formatting — Perl L63-120
  // These customize section numbering and font for Elsevier style
  def_macro_noop("\\elsartstyle")?;
  def_macro_noop("\\semark{}")?;
  def_macro_noop("\\ssmark{}")?;
  def_macro_noop("\\sssmark{}")?;
  def_macro_noop("\\elsmarks")?;

  // Abstract keywords with continuation
  DefMacro!("\\KWD{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  DefMacro!("\\AMS{}",  "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}");
  DefMacro!("\\PAC{}",  "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}");

  // Theorem environments — Perl L69-91
  // Perl L69-91: the full list of elsart theorem environments
  RawTeX!("\\theoremstyle{plain}");
  RawTeX!("\\@ifundefined{cor}{\\newtheorem{cor}[thm]{Corollary}}{}");
  RawTeX!("\\@ifundefined{lem}{\\newtheorem{lem}[thm]{Lemma}}{}");
  RawTeX!("\\@ifundefined{claim}{\\newtheorem{claim}[thm]{Claim}}{}");
  RawTeX!("\\@ifundefined{axiom}{\\newtheorem{axiom}[thm]{Axiom}}{}");
  RawTeX!("\\@ifundefined{conj}{\\newtheorem{conj}[thm]{Conjecture}}{}");
  RawTeX!("\\@ifundefined{fact}{\\newtheorem{fact}[thm]{Fact}}{}");
  RawTeX!("\\@ifundefined{hypo}{\\newtheorem{hypo}[thm]{Hypothesis}}{}");
  RawTeX!("\\@ifundefined{assum}{\\newtheorem{assum}[thm]{Assumption}}{}");
  RawTeX!("\\@ifundefined{prop}{\\newtheorem{prop}[thm]{Proposition}}{}");
  RawTeX!("\\@ifundefined{crit}{\\newtheorem{crit}[thm]{Criterion}}{}");
  RawTeX!("\\theoremstyle{definition}");
  RawTeX!("\\@ifundefined{defn}{\\newtheorem{defn}[thm]{Definition}}{}");
  RawTeX!("\\@ifundefined{exmp}{\\newtheorem{exmp}[thm]{Example}}{}");
  RawTeX!("\\@ifundefined{rem}{\\newtheorem{rem}[thm]{Remark}}{}");
  RawTeX!("\\@ifundefined{prob}{\\newtheorem{prob}[thm]{Problem}}{}");
  RawTeX!("\\@ifundefined{prin}{\\newtheorem{prin}[thm]{Principle}}{}");
  RawTeX!("\\@ifundefined{alg}{\\newtheorem{alg}{Algorithm}}{}");
  RawTeX!("\\@ifundefined{note}{\\newtheorem{note}{Note}}{}");
  RawTeX!("\\@ifundefined{summ}{\\newtheorem{summ}{Summary}}{}");
  RawTeX!("\\@ifundefined{case}{\\newtheorem{case}{Case}}{}");

  // Nuclear isotopes — Perl L60-65
  DefMacro!("\\nuc{}{}", "\\ensuremath{{}^{#2}\\mathrm{#1}}");
  DefMacro!("\\itnuc{}{}", "\\ensuremath{{}^{#2}\\textit{#1}}");

  // Perl elsart_support.sty.ltxml L63-65: \@@nuc — internal DefConstructor
  // that \nuc and \itnuc forward through in Perl. Rust short-circuits
  // \nuc/\itnuc above, so adding \@@nuc is purely defensive — external
  // code or Let-aliases that call \@@nuc{element}{mass} directly now
  // resolve to the Perl-faithful XMArg/XMApp wrapper with role=
  // SUPERSCRIPTOP. Simplification: Perl's properties closure computes
  // pos='pre<scriptlevel>' for pre-superscript positioning; we emit the
  // wrap unconditionally (position is determined by surrounding XMath).
  DefConstructor!("\\@@nuc{}{}",
    "<ltx:XMArg><ltx:XMApp>\
       <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='pre'/>#1#2\
     </ltx:XMApp></ltx:XMArg>");

  // Perl L92-102: algorithm counter + environment
  NewCounter!("algorithm");
  DefMacro!("\\thealgorithm", "\\arabic{algorithm}");
  DefMacro!("\\algorithmname", "Algorithm");
  // Perl L96-102: {algorithm} env. Was unported — \begin{algorithm}
  // hit an undefined-env error in any Elsevier paper. Rendered as a
  // <ltx:theorem> with class ltx_theorem_algorithm + float
  // numbering. Closing tag elided in the template; before/after
  // float hooks attach number/id; after_construct closes the
  // ltx:theorem at paragraph boundary (matches Perl's
  // maybeCloseElement).
  DefEnvironment!("{algorithm}",
    "<ltx:theorem xml:id='#id' class='ltx_theorem_algorithm'>#tags#body</ltx:theorem>",
    mode => "internal_vertical",
    before_digest => {
      use crate::engine::latex_constructs::before_float;
      before_float("algorithm", None);
    },
    after_digest => sub[whatsit] {
      use crate::engine::latex_constructs::after_float;
      after_float(whatsit);
    }
  );

  // Perl L104: \pf proof environment
  RawTeX!("\\@ifundefined{pf}{\\newenvironment{pf}{\\begin{@proof}[\\proofname]}{\\end{@proof}}}{}");

  // Caption continuations — Perl L108-110
  DefMacro!("\\contcaption", "\\caption{continued}");
  DefMacro!("\\contfigurecaption", "\\caption{continued}");
  DefMacro!("\\conttablecaption", "\\caption{continued}");

  // Bibliography — Perl L117-175
  DefEnvironment!("{subbibitems}", "#body");

  // Perl elsart_support.sty.ltxml L120: `{cv*}` env wraps its body in a
  // <ltx:section class='ltx_cv'> with auto-title "Curriculum Vitae".
  // Used in Elsevier journal submissions that include author CVs as a
  // tail section. Rust had only the non-env `\cv` DefMacro below;
  // `\begin{cv*}...\end{cv*}` hit undefined-env.
  DefEnvironment!("{cv*}",
    "<ltx:section class='ltx_cv'><ltx:title>Curriculum Vitae</ltx:title>#body</ltx:section>");

  def_macro_noop("\\cv")?;
  def_macro_noop("\\biboptions{}")?;
  def_macro_noop("\\bibliographystyle{}")?;
  DefMacro!("\\harvarditem[]{}{}{}",
    "\\bibitem[#2(#3)]{#4}");
  DefMacro!("\\harvardand", "\\&");
  DefMacro!("\\harvardurl{}", "\\url{#1}");
  // \harvestremark{text} carries author-typed remark in harvard
  // bibliography style. Surpass Perl gobble — preserve as note.
  DefMacro!("\\harvestremark{}",
    "\\@add@frontmatter{ltx:note}[role=harvestremark]{#1}");
  DefMacro!("\\harvardyearleft", "(");
  DefMacro!("\\harvardyearright", ")");
  def_macro_noop("\\citestyle{}")?;

  // Shorthands — Perl L124-128
  DefMacro!("\\AND", "\\&");
  DefMacro!("\\etal", "et al.");
  DefMacro!("\\Elproofname", "Proof.");
  DefMacro!("\\proofname", "Proof.");

  // Dimensions — Perl L132-139
  DefMacro!("\\cropwidth", "297mm");
  DefMacro!("\\cropheight", "210mm");
  DefMacro!("\\cropleft", "0mm");
  DefMacro!("\\croptop", "0mm");
  DefRegister!("\\rulepreskip" => Dimension!("4pt"));
  def_macro_noop("\\setleftmargin{}{}")?;

  // Misc — Perl L143-175
  Let!("\\realpageref", "\\pageref");
  def_macro_noop("\\snm")?;

  // Perl L146-156: \xalph / \xarabic / \xfnsymbol — emit * for negative counter, else
  // delegate to \alph / \arabic / \fnsymbol.
  DefMacro!("\\xalph{}", sub[(ctr)] {
    let n = counter_value(&ctr.to_string()).map(|c| c.value_of()).unwrap_or(0);
    if n < 0 {
      Ok(Tokens!(T_OTHER!("*")))
    } else {
      Ok(Tokens!(T_CS!("\\alph"), T_BEGIN!(), ctr, T_END!()))
    }
  });
  DefMacro!("\\xarabic{}", sub[(ctr)] {
    let n = counter_value(&ctr.to_string()).map(|c| c.value_of()).unwrap_or(0);
    if n < 0 {
      Ok(Tokens!(T_OTHER!("*")))
    } else {
      Ok(Tokens!(T_CS!("\\arabic"), T_BEGIN!(), ctr, T_END!()))
    }
  });
  DefMacro!("\\xfnsymbol{}", sub[(ctr)] {
    let n = counter_value(&ctr.to_string()).map(|c| c.value_of()).unwrap_or(0);
    if n < 0 {
      Ok(Tokens!(T_OTHER!("*")))
    } else {
      Ok(Tokens!(T_CS!("\\fnsymbol"), T_BEGIN!(), ctr, T_END!()))
    }
  });

  DefEnvironment!("{NoHyper}", "#body");
  def_macro_noop("\\mpfootnotemark")?;
  // Perl L162-167: \FMSlash/\FMslash overstrike / through content
  DefMacro!("\\FMSlash", "\\protect\\pFMSlash");
  DefMacro!("\\FMslash", "\\protect\\pFMslash");
  DefMacro!("\\pFMSlash{}", "#1\\Slashbox");
  DefMacro!("\\pFMslash{}", "#1\\slashbox");
  DefMacro!("\\Slashbox", "/");
  DefMacro!("\\slashbox", "/");

  // Perl elsart_support.sty.ltxml L172:
  //   DefMacro('\note{}', "<ltx:note>#1</ltx:note>");    # ?
  //
  // That `# ?` marks the author's uncertainty — a DefMacro expansion body
  // is a token stream (so `<`, `l`, `t`, `x`, `:`, `n`, `o`, `t`, `e`, `>`
  // are 10 OTHER tokens, not an ltx:note open tag). The Rust port uses
  // DefConstructor to emit a proper `<ltx:note>` element — matches the
  // clear intent of the Perl source and what actually renders. Kept as
  // an intentional Rust-over-Perl fix; the DP audit mismatch is expected.
  DefConstructor!("\\note{}", "<ltx:note>#1</ltx:note>");
  // \query{text} is author-typed editorial query. Preserve as note.
  DefMacro!("\\query{}",
    "\\@add@frontmatter{ltx:note}[role=query]{#1}");
});
