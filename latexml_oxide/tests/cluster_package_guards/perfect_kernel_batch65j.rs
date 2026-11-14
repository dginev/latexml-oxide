//! Red/green guards for batch 65j: a `\left`, `\middle` or `\right` without its partner in a `\bm` argument stays at
//! the group level `\bm` is used at, as bm.sty's token scan (bm.sty:240-305) emits it, its delimiter bold
//! (`\bm@delimiter`, :416); a pair inside one argument stays the bounded `\lx@bm@bold`.
//! Since 65d (OXIDIZED_DESIGN #530) `\left` opens a `math_left_group`; the bounded `\bm` digested its argument in a
//! group of its own, whose end closed that group: witness 2204.13665 (`\bm{\left\langle} #1 \bm{\right\rangle}`,
//! 0 -> 587 errors). OXIDIZED_DESIGN #544. Repro `tools/perfect_kernel/repros/boxes-groups/
//! bm_left_right_at_the_outer_level.tex`.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` under ar5iv, which must convert with no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// A document loading bm with `body` as its paragraph.
fn bm_doc(body: &str) -> String {
  format!(
    "\\documentclass{{article}}\n\\usepackage{{bm}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
  )
}

/// 2204.13665's `\lin`: a bold ⟨ in one `\bm`, a bold ⟩ in another, `x` between them neither bold nor grouped. The
/// three are one fenced formula. Base (65d): "\bm Attempt to close a group that switched to mode math", ⟩ lost.
#[test]
fn bm_left_and_right_in_separate_arguments() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bm_left_right_at_the_outer_level.tex"
  ));
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\left\langle x\right\rangle" text="delimited-⟨⟩@(x)" xml:id="p1.m1">
      <XMath xml:id="p1.m1.2">
        <XMDual xml:id="p1.m1.2.1">
          <XMApp xml:id="p1.m1.2.1.1">
            <XMTok meaning="delimited-⟨⟩"/>
            <XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.1.2"/>
          </XMApp>
          <XMWrap xml:id="p1.m1.2.1.2">
            <XMTok font="bold" name="langle" role="OPEN" stretchy="true">⟨</XMTok>
            <XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok>
            <XMTok font="bold" name="rangle" role="CLOSE" stretchy="true">⟩</XMTok>
          </XMWrap>
        </XMDual>
      </XMath>
    </Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m4""#],
    r#"<Math mode="inline" tex="\bm{x}" text="x" xml:id="p1.m4">
      <XMath xml:id="p1.m4.1">
        <XMTok font="bold italic" role="UNKNOWN">x</XMTok>
      </XMath>
    </Math>"#,
  );
}

/// `\bm{\left( a \right)}`: a pair inside one argument stays the bounded `\lx@bm@bold`, byte-identical to the 65 bundle
/// (the `math_left_group` opens and closes inside its group); the fence and `a` bold.
#[test]
fn bm_left_right_in_one_argument() {
  let xml = convert_clean(&bm_doc(r"$\bm{\left( a \right)}$"));
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\bm{\left(a\right)}" text="a" xml:id="p1.m1">
      <XMath xml:id="p1.m1.2">
        <XMDual xml:id="p1.m1.2.1">
          <XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.1"/>
          <XMWrap xml:id="p1.m1.2.1.2">
            <XMTok font="bold" role="OPEN" stretchy="true">(</XMTok>
            <XMTok font="bold italic" role="UNKNOWN" xml:id="p1.m1.1">a</XMTok>
            <XMTok font="bold" role="CLOSE" stretchy="true">)</XMTok>
          </XMWrap>
        </XMDual>
      </XMath>
    </Math>"#,
  );
}

/// A script after a paired `\right` (`\bm{\left(A+B\right)^\mathsf{T}}`) stays on the bold fenced whole: the pair keeps
/// the argument bounded. Round 2 split it and lost the superscript.
#[test]
fn bm_paired_left_right_keeps_its_superscript() {
  let xml = convert_clean(&bm_doc(r"$\bm{\left(A+B\right)^\mathsf{T}}$"));
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\bm{\left(A+B\right)^{\mathsf{T}}}" text="(A + B) ^ T" xml:id="p1.m1">
      <XMath xml:id="p1.m1.2">
        <XMApp xml:id="p1.m1.2.1">
          <XMTok role="SUPERSCRIPTOP" scriptpos="post2"/>
          <XMDual xml:id="p1.m1.2.1.2">
            <XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.2.1"/>
            <XMWrap xml:id="p1.m1.2.1.2.2">
              <XMTok font="bold" role="OPEN" stretchy="true">(</XMTok>
              <XMApp xml:id="p1.m1.1">
                <XMTok font="bold" meaning="plus" role="ADDOP">+</XMTok>
                <XMTok font="bold italic" role="UNKNOWN">A</XMTok>
                <XMTok font="bold italic" role="UNKNOWN">B</XMTok>
              </XMApp>
              <XMTok font="bold" role="CLOSE" stretchy="true">)</XMTok>
            </XMWrap>
          </XMDual>
          <XMTok font="sansserif bold" fontsize="70%" role="UNKNOWN">T</XMTok>
        </XMApp>
      </XMath>
    </Math>"#,
  );
}

/// `\bm{\middle|}` inside a `\left`…`\right`: a bold MIDDLE bar in the set. bm.sty bolds a `\middle` delimiter given as
/// `\delimiter` (`\vert`) but takes a character after `\middle` for a math character (its `\ifx\bm@previous\left`,
/// :289, knows `\left` and `\right` only), so pdflatex reports "Missing delimiter (. inserted)" for `\middle|`; the bar
/// is kept as the bold delimiter the author wrote, as Perl and 65a do. OXIDIZED_DESIGN #544.
#[test]
fn bm_middle_bar() {
  let xml = convert_clean(&bm_doc(r"$\left\{ y \bm{\middle|} y > 0 \right\}$"));
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\left\{y\middle|y&gt;0\right\}" text="conditional-set@(y, y &gt; 0)" xml:id="p1.m1">
      <XMath xml:id="p1.m1.3">
        <XMDual xml:id="p1.m1.3.1">
          <XMApp xml:id="p1.m1.3.1.1">
            <XMTok meaning="conditional-set"/>
            <XMRef idref="p1.m1.1" xml:id="p1.m1.3.1.1.2"/>
            <XMRef idref="p1.m1.2" xml:id="p1.m1.3.1.1.3"/>
          </XMApp>
          <XMWrap xml:id="p1.m1.3.1.2">
            <XMTok role="OPEN" stretchy="true">{</XMTok>
            <XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">y</XMTok>
            <XMTok font="bold" role="MIDDLE" role_side="middle" stretchy="true">|</XMTok>
            <XMApp xml:id="p1.m1.2">
              <XMTok meaning="greater-than" role="RELOP">&gt;</XMTok>
              <XMTok font="italic" role="UNKNOWN">y</XMTok>
              <XMTok meaning="0" role="NUMBER">0</XMTok>
            </XMApp>
            <XMTok role="CLOSE" stretchy="true">}</XMTok>
          </XMWrap>
        </XMDual>
      </XMath>
    </Math>"#,
  );
}

/// The control: a `\bm` without a delimiter command is the bounded bold constructor, as before (byte-identical to the
/// 65 bundle).
#[test]
fn bm_without_delimiters_is_unchanged() {
  let xml = convert_clean(&bm_doc(r"$\bm{x} + \bm{\alpha_1} = \boldsymbol{(a+b)}$"));
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\bm{x}+\bm{\alpha_{1}}=\bm{(a+b)}" text="x + alpha _ 1 = a + b" xml:id="p1.m1">
      <XMath xml:id="p1.m1.2">
        <XMApp xml:id="p1.m1.2.1">
          <XMTok meaning="equals" role="RELOP">=</XMTok>
          <XMApp xml:id="p1.m1.2.1.2">
            <XMTok meaning="plus" role="ADDOP">+</XMTok>
            <XMTok font="bold italic" role="UNKNOWN">x</XMTok>
            <XMApp xml:id="p1.m1.2.1.2.3">
              <XMTok role="SUBSCRIPTOP" scriptpos="post2"/>
              <XMTok font="bold italic" name="alpha" role="UNKNOWN">α</XMTok>
              <XMTok font="bold" fontsize="70%" meaning="1" role="NUMBER">1</XMTok>
            </XMApp>
          </XMApp>
          <XMDual xml:id="p1.m1.2.1.3">
            <XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.3.1"/>
            <XMWrap xml:id="p1.m1.2.1.3.2">
              <XMTok font="bold" role="OPEN" stretchy="false">(</XMTok>
              <XMApp xml:id="p1.m1.1">
                <XMTok font="bold" meaning="plus" role="ADDOP">+</XMTok>
                <XMTok font="bold italic" role="UNKNOWN">a</XMTok>
                <XMTok font="bold italic" role="UNKNOWN">b</XMTok>
              </XMApp>
              <XMTok font="bold" role="CLOSE" stretchy="false">)</XMTok>
            </XMWrap>
          </XMDual>
        </XMApp>
      </XMath>
    </Math>"#,
  );
}

/// A `\delimiter` code in the argument (bm.sty's own route, `\bm@delimiter`): the number is the delimiter's, read with
/// it, not bolded apart (tex.web §440-445 at the token level, `take_number`); the null delimiter has no glyph to bold.
#[test]
fn bm_delimiter_codes_and_the_null_delimiter() {
  let (log, xml) = convert_with(
    &bm_doc(
      r#"$\bm{\left\delimiter"426830A} x \bm{\right\delimiter"5269309 }$ and $\bm{\left.} y \bm{\right]}$"#,
    ),
    Some("ar5iv.sty"),
  );
  // `. y ]` has no parse: the one warning is the parser's.
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(log.contains("Warning:unparsed_math:"), "{log}");

  let tokens: Vec<&str> = xml
    .split("<XMTok")
    .skip(1)
    .filter_map(|t| t.find("</XMTok>").map(|end| &t[..end]))
    .collect();
  assert_eq!(
    tokens,
    [
      r#" font="bold" name="langle" role="OPEN" stretchy="true">⟨"#,
      r#" font="italic" role="UNKNOWN" xml:id="p1.m1.1">x"#,
      r#" font="bold" name="rangle" role="CLOSE" stretchy="true">⟩"#,
      r#" font="italic" role="UNKNOWN">y"#,
      r#" font="bold" role="CLOSE" stretchy="true">]"#,
    ],
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<Math class="ltx_math_unparsed" mode="inline" tex="\left.y\right]" xml:id="p1.m2">"#
    ),
    "{xml}"
  );
}

/// `\bm` in text: Perl's `requireMath` warning, naming `\bm` (not the bounded constructor behind it), and bold text.
#[test]
fn bm_in_text_warns_naming_bm() {
  let (log, xml) = convert_with(&bm_doc(r"A \bm{t} b."), Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(
    log.contains(r"Warning:unexpected:\bm \bm should only appear in math mode"),
    "{log}"
  );
  assert!(
    xml.contains(r#"<text font="bold" xml:id="p1.1.1">t</text>"#),
    "{xml}"
  );
}

/// A script after the `\right` inside the argument (2107.08152 `\bm{\left\langle z \right\rangle_{L,R}}`, 1703.10645
/// `\bm{\left(\Sigma_i^e \right)_{(j,j)}}`): the pair keeps the argument the bounded `\lx@bm@bold`, so the subscript
/// stays on the fence, bold (byte-identical to the 65 bundle).
#[test]
fn bm_script_after_paired_right_stays_bounded() {
  let xml = convert_clean(&bm_doc(r"$\bm{\left\langle z \right\rangle_{L,R}}$"));
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\bm{\left\langle z\right\rangle_{L,R}}" text="(delimited-⟨⟩@(z)) _ (list@(L, R))" xml:id="p1.m1">
      <XMath xml:id="p1.m1.4">
        <XMApp xml:id="p1.m1.4.1">
          <XMTok role="SUBSCRIPTOP" scriptpos="post2"/>
          <XMDual xml:id="p1.m1.4.1.2">
            <XMApp xml:id="p1.m1.4.1.2.1">
              <XMTok meaning="delimited-⟨⟩"/>
              <XMRef idref="p1.m1.1" xml:id="p1.m1.4.1.2.1.2"/>
            </XMApp>
            <XMWrap xml:id="p1.m1.4.1.2.2">
              <XMTok font="bold" name="langle" role="OPEN" stretchy="true">⟨</XMTok>
              <XMTok font="bold italic" role="UNKNOWN" xml:id="p1.m1.1">z</XMTok>
              <XMTok font="bold" name="rangle" role="CLOSE" stretchy="true">⟩</XMTok>
            </XMWrap>
          </XMDual>
          <XMDual xml:id="p1.m1.4.1.3">
            <XMApp xml:id="p1.m1.4.1.3.1">
              <XMTok meaning="list"/>
              <XMRef idref="p1.m1.2" xml:id="p1.m1.4.1.3.1.2"/>
              <XMRef idref="p1.m1.3" xml:id="p1.m1.4.1.3.1.3"/>
            </XMApp>
            <XMWrap xml:id="p1.m1.4.1.3.2">
              <XMTok font="bold italic" fontsize="70%" role="UNKNOWN" xml:id="p1.m1.2">L</XMTok>
              <XMTok font="bold" fontsize="70%" role="PUNCT">,</XMTok>
              <XMTok font="bold italic" fontsize="70%" role="UNKNOWN" xml:id="p1.m1.3">R</XMTok>
            </XMWrap>
          </XMDual>
        </XMApp>
      </XMath>
    </Math>"#,
  );
}

/// A script after an unpaired `\right` whose argument is a command with its own argument (`_\mathrm{L}`): bm.sty
/// expands the argument before it bolds it (`\bm@expand`), so `\mathrm{L}` stays whole — one bold upright L on the
/// fence. Round 2 cut `\mathrm` from `{L}`: `_{\bm{\mathrm{}}}\bm{{L}}`.
#[test]
fn bm_script_command_argument_after_unpaired_right() {
  let xml = convert_clean(&bm_doc(
    r"$\bm{\left\langle} x \bm{\right\rangle_\mathrm{L}}$",
  ));
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\left\langle x\right\rangle_{\bm{\mathrm{L}}}" text="(delimited-⟨⟩@(x)) _ L" xml:id="p1.m1">
      <XMath xml:id="p1.m1.2">
        <XMApp xml:id="p1.m1.2.1">
          <XMTok role="SUBSCRIPTOP" scriptpos="post1"/>
          <XMDual xml:id="p1.m1.2.1.2">
            <XMApp xml:id="p1.m1.2.1.2.1">
              <XMTok meaning="delimited-⟨⟩"/>
              <XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.2.1.2"/>
            </XMApp>
            <XMWrap xml:id="p1.m1.2.1.2.2">
              <XMTok font="bold" name="langle" role="OPEN" stretchy="true">⟨</XMTok>
              <XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok>
              <XMTok font="bold" name="rangle" role="CLOSE" stretchy="true">⟩</XMTok>
            </XMWrap>
          </XMDual>
          <XMTok font="bold" fontsize="70%" role="UNKNOWN">L</XMTok>
        </XMApp>
      </XMath>
    </Math>"#,
  );
}

/// A prime after an unpaired `\right` is a bold ′ on the fence (bm.sty:177 `\def'{\bm@prime\prime\relax}`).
#[test]
fn bm_prime_after_unpaired_right_is_bold() {
  let xml = convert_clean(&bm_doc(r"$\bm{\left(} x \bm{\right)'}$"));
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\left(x\right)^{\bm{\prime}}" text="x ^ prime" xml:id="p1.m1">
      <XMath xml:id="p1.m1.2">
        <XMApp xml:id="p1.m1.2.1">
          <XMTok role="SUPERSCRIPTOP" scriptpos="post1"/>
          <XMDual xml:id="p1.m1.2.1.2">
            <XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.2.1"/>
            <XMWrap xml:id="p1.m1.2.1.2.2">
              <XMTok font="bold" role="OPEN" stretchy="true">(</XMTok>
              <XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok>
              <XMTok font="bold" role="CLOSE" stretchy="true">)</XMTok>
            </XMWrap>
          </XMDual>
          <XMTok font="bold" fontsize="70%" name="prime" role="SUPOP">′</XMTok>
        </XMApp>
      </XMath>
    </Math>"#,
  );
}

/// A script argument that cannot be told without expanding it (a parameterless macro before a `{`): the rest of the
/// run is the script's argument, with one Warning naming `\bm`.
#[test]
fn bm_script_argument_that_cannot_be_told_warns() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{bm}
\newcommand\foo{\mathrm}
\begin{document}
$\bm{\left(} x \bm{\right)_\foo{L}}$
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(
    log.contains(
      r"Warning:unexpected:\bm Cannot tell the argument of _ (starting \foo) after a \left, \middle or \right in the argument of \bm; the rest of the argument is taken as it"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\left(x\right)_{\bm{\mathrm{L}}}" text="x _ L" xml:id="p1.m1">
      <XMath xml:id="p1.m1.2">
        <XMApp xml:id="p1.m1.2.1">
          <XMTok role="SUBSCRIPTOP" scriptpos="post1"/>
          <XMDual xml:id="p1.m1.2.1.2">
            <XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.2.1"/>
            <XMWrap xml:id="p1.m1.2.1.2.2">
              <XMTok font="bold" role="OPEN" stretchy="true">(</XMTok>
              <XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok>
              <XMTok font="bold" role="CLOSE" stretchy="true">)</XMTok>
            </XMWrap>
          </XMDual>
          <XMTok font="bold" fontsize="70%" role="UNKNOWN">L</XMTok>
        </XMApp>
      </XMath>
    </Math>"#,
  );
}

/// The paired `\left( a \right)` in one argument of a command `cs` that sets its argument in a box or group: one
/// bounded bold group, as before (byte-identical to the 65 bundle).
fn paired_in_one_group(cs: &str) -> String {
  format!(
    r#"<Math mode="inline" tex="{cs}{{\left(a\right)}}" text="a" xml:id="p1.m2">
      <XMath xml:id="p1.m2.2">
        <XMDual xml:id="p1.m2.2.1">
          <XMRef idref="p1.m2.1" xml:id="p1.m2.2.1.1"/>
          <XMWrap xml:id="p1.m2.2.1.2">
            <XMTok font="bold" role="OPEN" stretchy="true">(</XMTok>
            <XMTok font="bold italic" role="UNKNOWN" xml:id="p1.m2.1">a</XMTok>
            <XMTok font="bold" role="CLOSE" stretchy="true">)</XMTok>
          </XMWrap>
        </XMDual>
      </XMath>
    </Math>"#
  )
}

/// The repro `name`: `cs` (`\bm` of a class without bm.sty, or amsbsy's `\boldsymbol`) sets its argument in a box or
/// group, so `\lin{x}` = `cs{\left\langle} x cs{\right\rangle}` is TeX's error at the box's end ("Missing \right.
/// inserted", "Extra \right."). It is set at the outer level as bm.sty would set it — a bold ⟨ x ⟩ fenced in one
/// formula, `x` neither bold nor grouped — with one Warning per unpaired command; the paired `cs{\left( a \right)}`
/// stays one bounded group. Base (65 bundle, 65j round 1): "cs Attempt to close a group that switched to mode math",
/// ⟩ lost. OXIDIZED_DESIGN #544.
fn assert_grouped_bold_unpaired(name: &str, tex: &str, cs: &str) {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{name}: {log}");
  assert_eq!(warning_count(&log), 2, "{name}: {log}");
  let warning = format!(
    r#"Warning:unexpected:{cs} \left, \middle or \right without its partner in the argument of {cs}, whose box or group ends it in TeX ("Missing \right. inserted"); set at the outer level"#
  );
  assert_eq!(log.matches(&warning).count(), 2, "{name}: {log}");
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    &paired_in_one_group(cs),
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m3""#],
    r#"<Math mode="inline" tex="\left\langle x\right\rangle" text="delimited-⟨⟩@(x)" xml:id="p1.m3">
      <XMath xml:id="p1.m3.2">
        <XMDual xml:id="p1.m3.2.1">
          <XMApp xml:id="p1.m3.2.1.1">
            <XMTok meaning="delimited-⟨⟩"/>
            <XMRef idref="p1.m3.1" xml:id="p1.m3.2.1.1.2"/>
          </XMApp>
          <XMWrap xml:id="p1.m3.2.1.2">
            <XMTok font="bold" name="langle" role="OPEN" stretchy="true">⟨</XMTok>
            <XMTok font="italic" role="UNKNOWN" xml:id="p1.m3.1">x</XMTok>
            <XMTok font="bold" name="rangle" role="CLOSE" stretchy="true">⟩</XMTok>
          </XMWrap>
        </XMDual>
      </XMath>
    </Math>"#,
  );
}

/// RevTeX's `\bm` without bm.sty (Perl revtex4_support.sty.ltxml:167; revtex4-2.cls:7573-7586 asks for bm.sty).
#[test]
fn revtex_bm_unpaired_left_right_at_the_outer_level() {
  assert_grouped_bold_unpaired(
    "revtex4-2",
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/bm_revtex_unpaired_left_right.tex"
    ),
    r"\bm",
  );
}

/// The World Scientific classes' own `\bm` (ws-ijmpe.cls:226-230 `\mbox{\boldmath$#1$}`).
#[test]
fn ws_class_bm_unpaired_left_right_at_the_outer_level() {
  assert_grouped_bold_unpaired(
    "ws-ijmpc",
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/bm_ws_class_unpaired_left_right.tex"
    ),
    r"\bm",
  );
}

/// interact.cls's own `\bm` (interact.cls:134-138 `\mbox{\boldmath$\displaystyle#1$}`).
#[test]
fn interact_bm_unpaired_left_right_at_the_outer_level() {
  assert_grouped_bold_unpaired(
    "interact",
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/bm_interact_unpaired_left_right.tex"
    ),
    r"\bm",
  );
}

/// amsbsy's `\boldsymbol` and `\pmb` (Let to it, Perl amsbsy.sty.ltxml:28), boxed (amsbsy.sty:28-37).
#[test]
fn boldsymbol_unpaired_left_right_at_the_outer_level() {
  assert_grouped_bold_unpaired(
    "amsbsy",
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/boldsymbol_unpaired_left_right.tex"
    ),
    r"\boldsymbol",
  );
}

/// The control: amsbsy's `\boldsymbol` without an unpaired delimiter command is the bounded constructor reverting as
/// `\boldsymbol` — a script after a paired `\right` and a `\left(\right.` pair included — and in text it warns naming
/// `\boldsymbol` (byte-identical to the 65 bundle).
#[test]
fn boldsymbol_paired_or_without_delimiters_is_unchanged() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
$\pmb{\alpha_1} = \boldsymbol\beta$, text \boldsymbol{t} and $\boldsymbol{\left[ a \right]_{i}}$.
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(
    log.contains(r"Warning:unexpected:\boldsymbol \boldsymbol should only appear in math mode"),
    "{log}"
  );
  assert!(
    xml.contains(r#"<text font="bold" xml:id="p1.1.1">t</text>"#),
    "{xml}"
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\boldsymbol{\alpha_{1}}=\boldsymbol{\beta}" text="alpha _ 1 = beta" xml:id="p1.m1">
      <XMath xml:id="p1.m1.1">
        <XMApp xml:id="p1.m1.1.1">
          <XMTok meaning="equals" role="RELOP">=</XMTok>
          <XMApp xml:id="p1.m1.1.1.2">
            <XMTok role="SUBSCRIPTOP" scriptpos="post2"/>
            <XMTok font="bold italic" name="alpha" role="UNKNOWN">α</XMTok>
            <XMTok font="bold" fontsize="70%" meaning="1" role="NUMBER">1</XMTok>
          </XMApp>
          <XMTok font="bold italic" name="beta" role="UNKNOWN">β</XMTok>
        </XMApp>
      </XMath>
    </Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="\boldsymbol{\left[a\right]_{i}}" text="(delimited-[]@(a)) _ i" xml:id="p1.m2">
      <XMath xml:id="p1.m2.2">
        <XMApp xml:id="p1.m2.2.1">
          <XMTok role="SUBSCRIPTOP" scriptpos="post2"/>
          <XMDual xml:id="p1.m2.2.1.2">
            <XMApp xml:id="p1.m2.2.1.2.1">
              <XMTok meaning="delimited-[]"/>
              <XMRef idref="p1.m2.1" xml:id="p1.m2.2.1.2.1.2"/>
            </XMApp>
            <XMWrap xml:id="p1.m2.2.1.2.2">
              <XMTok font="bold" role="OPEN" stretchy="true">[</XMTok>
              <XMTok font="bold italic" role="UNKNOWN" xml:id="p1.m2.1">a</XMTok>
              <XMTok font="bold" role="CLOSE" stretchy="true">]</XMTok>
            </XMWrap>
          </XMDual>
          <XMTok font="bold italic" fontsize="70%" role="UNKNOWN">i</XMTok>
        </XMApp>
      </XMath>
    </Math>"#,
  );
}
