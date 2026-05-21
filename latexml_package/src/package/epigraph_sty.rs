use crate::prelude::*;


LoadDefinitions!({
  TeX!(
    r#"""
\newlength{\beforeepigraphskip}
  \setlength{\beforeepigraphskip}{.5\baselineskip}
\newlength{\afterepigraphskip}
  \setlength{\afterepigraphskip}{.5\baselineskip}
\newlength{\epigraphwidth}
  \setlength{\epigraphwidth}{.4\textwidth}
\newlength{\epigraphrule}
  \setlength{\epigraphrule}{.4\p@}
\newcommand{\epigraphsize}{\small}
\newcommand{\epigraphflush}{flushright}
\newcommand{\textflush}{flushleft}
\newcommand{\sourceflush}{flushright}
"""#
  );

  DefConstructor!("\\epigraph{}{}",
    "<ltx:quote class='ltx_epigraph #epigraphflush' cssstyle='#qwidth #qalign'>\
      <ltx:block class='ltx_epigraph_text' cssstyle='#talign'>#1</ltx:block>\
      <ltx:block class='ltx_epigraph_source' cssstyle='#srule #salign'>#2</ltx:block>\
    </ltx:quote>",
    bounded => true,
    before_digest => {
      stomach::digest(Tokens!(T_CS!("\\epigraphsize")))?;
    },
    after_digest => sub[whatsit] {
      let rule = LookupRegisterOrDefault!("\\epigraphrule");
      let rule_pt = match rule {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 0.4,
      };
      let width = LookupRegisterOrDefault!("\\epigraphwidth");
      let width_pt = match width {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 0.0,
      };
      let qa = gullet::do_expand(T_CS!("\\epigraphflush"))?.to_string();
      let ta = gullet::do_expand(T_CS!("\\textflush"))?.to_string();
      let sa = gullet::do_expand(T_CS!("\\sourceflush"))?.to_string();

      let qalign = match qa.as_str() {
        "center" => "margin-right:auto; margin-left:auto;",
        "flushleft" => "margin-right:auto;",
        "flushright" => "margin-left:auto;",
        _ => "",
      };
      let talign = match ta.as_str() {
        "center" => "text-align:center; ",
        "flushleft" => "text-align:left; ",
        "flushright" => "text-align:right; ",
        _ => "",
      };
      let salign = match sa.as_str() {
        "center" => "text-align:center; ",
        "flushleft" => "text-align:left; ",
        "flushright" => "text-align:right; ",
        _ => "",
      };

      whatsit.set_property("srule", s!("border-top:solid {}pt;", rule_pt));
      whatsit.set_property("qwidth", s!("width:{}pt;", width_pt));
      whatsit.set_property("qalign", qalign);
      whatsit.set_property("talign", talign);
      whatsit.set_property("salign", salign);
      // Note: do NOT set "epigraphflush" property — the template #epigraphflush
      // should resolve to empty (Perl never sets this property)
    }
  );

  DefEnvironment!("{epigraphs}", "#body",
    before_digest => {
      Let!("\\qitem", "\\epigraph");
    }
  );

  DefMacro!("\\epigraphhead[]{}", "#1");
  def_macro_noop("\\dropchapter{}")?;
  def_macro_noop("\\undodrop")?;
  def_macro_noop("\\cleartoevenpage")?;
});
