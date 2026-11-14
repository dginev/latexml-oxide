//! Red/green guards for batch 66g: the end of an input closes only a `\left` group opened in that input (stomach.rs
//! `close_left_group_at_input_end`, the reading context recorded by `\lx@math@left@group`; OXIDIZED_DESIGN #530). A
//! binding's `\left( \lx@xmarg{…}{#1} \right)` digests its argument from a mouth of its own, and that argument's end
//! closed the binding's own `\left` since 65d (sandbox 2605/2606 regression C5: 2605.02768, 2606.01486). Repros
//! `tools/perfect_kernel/repros/boxes-groups/{left_outside_an_argument_survives_its_end,
//! left_outside_an_argument_in_aligned,left_inside_an_argument_closes_at_its_end}.tex`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// `xml` from the first `open` to the end of its `close`, each line trimmed and the `xml:id`s dropped: a whole element,
/// indentation- and id-free.
fn element(xml: &str, open: &str, close: &str) -> String {
  let start = xml.find(open).unwrap_or_else(|| panic!("no {open}\n{xml}"));
  let end = xml[start..].find(close).map_or_else(
    || panic!("no {close} after {open}\n{xml}"),
    |e| start + e + close.len(),
  );
  xml[start..end]
    .lines()
    .map(|line| {
      let line = line.trim();
      match line.find(" xml:id=\"") {
        Some(at) => {
          let rest = &line[at + 9..];
          let close = rest.find('"').map_or(rest.len(), |q| q + 1);
          format!("{}{}", &line[..at], &rest[close..])
        },
        None => line.to_string(),
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// The `Warning:` lines of a log, ANSI-free.
fn warning_lines(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|l| l.find("Warning:").map(|i| &l[i..]))
    .collect()
}

/// 2605.02768 (SYK-HT-arxiv.tex:568 `\log(\cosh(\beta\lambda\over 2))`): physics.sty's `\cosh(…)` sets
/// `\left(#1\right)`; the binding digests `#1` in `\lx@xmarg`. Before: "Missing \right. inserted" at the argument's
/// end, then "Extra \right." and a dangling XMRef. The fraction stands between the binding's own delimiters.
#[test]
fn left_outside_an_argument_survives_its_end() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/left_outside_an_argument_survives_its_end.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    xml.contains(r#"<Math mode="inline" tex="\cosh(a\over 2)" text="hyperbolic-cosine@(a / 2)""#),
    "{xml}"
  );
  assert_eq!(
    element(&xml, "<XMDual", "</XMDual>"),
    r#"<XMDual>
<XMApp>
<XMTok meaning="hyperbolic-cosine"/>
<XMRef idref="p1.m1.2"/>
</XMApp>
<XMApp>
<XMTok role="OPFUNCTION" scriptpos="post">cosh</XMTok>
<XMWrap>
<XMTok role="OPEN" stretchy="true">(</XMTok>
<XMApp>
<XMTok mathstyle="text" meaning="divide" role="FRACOP"/>
<XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok>
<XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok>
</XMApp>
<XMTok role="CLOSE" stretchy="true">)</XMTok>
</XMWrap>
</XMApp>
</XMDual>"#,
    "{xml}"
  );
  // The XMRef points at the fraction, the argument's own node.
  assert!(xml.contains(r#"<XMApp xml:id="p1.m1.2">"#), "{xml}");
}

/// 2606.01486 (approxHFsym_dualities_v1.tex:446): physics' `\expval{\del D \over \del\Phi}_F` in an `aligned` row.
/// Before: "\lx@hidden@egroup Attempt to close boxing group" (the binding's `\right\rangle` met the row's
/// `\begingroup`) and the equation lost. Both rows are kept, the first one's expectation value whole.
#[test]
fn left_outside_an_argument_in_aligned() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/left_outside_an_argument_in_aligned.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    xml.contains(
      r#"<Math tex="\displaystyle a=b-\expectationvalue{\delta D \over\delta\Phi}_{F}" text="a = b - (expectation-value@((delta * D) / (delta * Phi))) _ F" xml:id="S0.E1X.m1">"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<Math tex="\displaystyle=c" text="absent = c" xml:id="S0.E1Xa.m1">"#),
    "{xml}"
  );
  assert_eq!(
    element(&xml, "<XMDual", "</XMDual>"),
    r#"<XMDual>
<XMApp>
<XMTok meaning="expectation-value"/>
<XMRef idref="S0.E1X.m1.1"/>
</XMApp>
<XMWrap>
<XMTok name="langle" role="OPEN" stretchy="true">⟨</XMTok>
<XMApp>
<XMTok mathstyle="display" meaning="divide" role="FRACOP"/>
<XMApp>
<XMTok meaning="times" role="MULOP">⁢</XMTok>
<XMTok font="italic" name="delta" role="UNKNOWN">δ</XMTok>
<XMTok font="italic" role="UNKNOWN">D</XMTok>
</XMApp>
<XMApp>
<XMTok meaning="times" role="MULOP">⁢</XMTok>
<XMTok font="italic" name="delta" role="UNKNOWN">δ</XMTok>
<XMTok name="Phi" role="UNKNOWN">Φ</XMTok>
</XMApp>
</XMApp>
<XMTok name="rangle" role="CLOSE" stretchy="true">⟩</XMTok>
</XMWrap>
</XMDual>"#,
    "{xml}"
  );
}

/// The control: a `\left` read inside the binding's argument is closed at the argument's end (65d's recovery, "Missing
/// \right. inserted"), while the binding's own `\left(` read outside it pairs with its `\right)`.
#[test]
fn left_inside_an_argument_closes_at_its_end() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/left_inside_an_argument_closes_at_its_end.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(
    warning_lines(&log),
    ["Warning:expected:\\right Missing \\right. inserted"],
    "{log}"
  );
  assert_eq!(
    element(&xml, "<XMDual", "</XMDual>"),
    r#"<XMDual>
<XMApp>
<XMTok meaning="hyperbolic-cosine"/>
<XMRef idref="p1.m1.1"/>
</XMApp>
<XMWrap>
<XMTok role="OPFUNCTION" scriptpos="post">cosh</XMTok>
<XMTok role="OPEN" stretchy="true">(</XMTok>
<XMArg>
<XMTok role="OPEN" stretchy="true">[</XMTok>
<XMTok font="italic" role="UNKNOWN">a</XMTok>
</XMArg>
<XMTok role="CLOSE" stretchy="true">)</XMTok>
</XMWrap>
</XMDual>"#,
    "{xml}"
  );
  assert!(xml.contains("</Math> after.</p>"), "{xml}");
}
