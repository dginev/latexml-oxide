use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl ar5iv-bindings/scrpage.sty.ltxml — KOMA Script page layout.
  // scrpage.sty is page-layout only; since LaTeXML targets HTML/MathML
  // where page layout is irrelevant, all 102 directives are stubbed to
  // either empty tokens, static length strings, or simple passthroughs.
  // Generated from Perl via /tmp/gen_scrpage.pl; hand-added the three
  // non-DefMacro directives missed by the generator
  // (DefConditional \if@autooneside Perl L65, DefConditional \if@chapter
  // Perl L86, Let \headmark \relax Perl L91).
  DefMacro!("\\scr@headabove@linethickness", "0pt");
  DefMacro!("\\scr@headbelow@linethickness", "0pt");
  DefMacro!("\\scr@footabove@linethickness", "0pt");
  DefMacro!("\\scr@footbelow@linethickness", "0pt");
  DefMacro!("\\scr@headabove@linelength", "\\@headwidth");
  DefMacro!("\\scr@headbelow@linelength", "\\@headwidth");
  DefMacro!("\\scr@footabove@linelength", "\\@footwidth");
  DefMacro!("\\scr@footbelow@linelength", "\\@footwidth");
  DefMacro!("\\scrplain@headabove@linelength", "0pt");
  DefMacro!("\\scrplain@headbelow@linelength", "0pt");
  DefMacro!("\\scrplain@footabove@linelength", "0pt");
  DefMacro!("\\scrplain@footbelow@linelength", "0pt");
  DefMacro!("\\KOMAScript", "\\textsf{KOMA Script}");
  DefMacro!("\\hfline@adjust", "1");
  DefConditional!("\\if@autooneside");
  def_macro_noop("\\scr@nouppercase")?;
  DefConditional!("\\if@chapter");
  Let!("\\headmark", "\\relax");
  DefMacro!("\\pagemark", "\\thepage");
  def_macro_noop("\\defpagestyle OptionalMatch:* {}{}{}")?;
  def_macro_noop("\\@defpagestyle[]{}{}{}")?;
  def_macro_noop("\\newpagestyle OptionalMatch:* {}{}{}")?;
  def_macro_noop("\\@newpagestyle[]{}{}{}")?;
  def_macro_noop("\\renewpagestyle OptionalMatch:* {}{}{}")?;
  def_macro_noop("\\@renewpagestyle[]{}{}{}")?;
  def_macro_noop("\\providepagestyle OptionalMatch:* {}{}{}")?;
  def_macro_noop("\\@providepagestyle[]{}{}{}")?;
  def_macro_noop("\\deftripstyle OptionalMatch:* {}")?;
  def_macro_noop("\\@deftripstyle[]{}")?;
  def_macro_noop("\\markleft{}")?;
  def_macro_noop("\\automark{}{}")?;
  def_macro_noop("\\manualmark")?;
  DefMacro!("\\chapterlevel", "0");
  DefMacro!("\\sectionlevel", "1");
  DefMacro!("\\subsectionlevel", "2");
  DefMacro!("\\subsubsectionlevel", "3");
  DefMacro!("\\paragraphlevel", "4");
  DefMacro!("\\subparagraphlevel", "5");
  def_macro_noop("\\settowidthof{}{}")?;
  def_macro_noop("\\deftowidthof{}{}")?;
  def_macro_noop("\\setheadwidth{}{}")?;
  def_macro_noop("\\setfootwidth{}{}")?;
  def_macro_noop("\\set@hf@width{}{}{}")?;
  DefMacro!("\\@headwidth", "\\textwidth");
  DefMacro!("\\@oddheadshift", "\\z@");
  DefMacro!("\\@evenheadshift", "\\z@");
  DefMacro!("\\@footwidth", "\\textwidth");
  DefMacro!("\\@oddfootshift", "\\z@");
  DefMacro!("\\@evenfootshift", "\\z@");
  DefMacro!("\\pnumfont", "\\normalfont");
  DefMacro!("\\headfont", "\\normalfont\\slshape");
  DefMacro!("\\setheadtopline", "\\scr@setline{head}{above}");
  DefMacro!("\\setheadsepline", "\\scr@setline{head}{below}");
  DefMacro!("\\setfootsepline", "\\scr@setline{foot}{above}");
  DefMacro!("\\setfootbotline", "\\scr@setline{foot}{below}");
  def_macro_noop("\\scr@setline OptionalMatch:* {}{}")?;
  def_macro_noop("\\scrplain@even@left@head")?;
  def_macro_noop("\\scrplain@even@middle@head")?;
  def_macro_noop("\\scrplain@even@right@head")?;
  def_macro_noop("\\scrplain@odd@left@head")?;
  def_macro_noop("\\scrplain@odd@middle@head")?;
  def_macro_noop("\\scrplain@odd@right@head")?;
  def_macro_noop("\\scrplain@even@left@foot")?;
  def_macro_noop("\\scrplain@even@middle@foot")?;
  def_macro_noop("\\scrplain@even@right@foot")?;
  def_macro_noop("\\scrplain@odd@left@foot")?;
  def_macro_noop("\\scrplain@odd@middle@foot")?;
  def_macro_noop("\\scrplain@odd@right@foot")?;
  DefMacro!("\\ps@scrheadings", "\\let\\ps@plain\\ps@scrplain\\ps@@scrheadings");
  def_macro_noop("\\scrheadings@even@left@head")?;
  def_macro_noop("\\scrheadings@even@middle@head")?;
  def_macro_noop("\\scrheadings@even@right@head")?;
  def_macro_noop("\\scrheadings@odd@left@head")?;
  def_macro_noop("\\scrheadings@odd@middle@head")?;
  def_macro_noop("\\scrheadings@odd@right@head")?;
  def_macro_noop("\\scrheadings@even@left@foot")?;
  def_macro_noop("\\scrheadings@even@middle@foot")?;
  def_macro_noop("\\scrheadings@even@right@foot")?;
  def_macro_noop("\\scrheadings@odd@left@foot")?;
  def_macro_noop("\\scrheadings@odd@middle@foot")?;
  def_macro_noop("\\scrheadings@odd@right@foot")?;
  def_macro_noop("\\ihead[]{}")?;
  def_macro_noop("\\ohead[]{}")?;
  def_macro_noop("\\chead[]{}")?;
  def_macro_noop("\\lehead[]{}")?;
  def_macro_noop("\\lohead[]{}")?;
  def_macro_noop("\\rehead[]{}")?;
  def_macro_noop("\\rohead[]{}")?;
  def_macro_noop("\\cehead[]{}")?;
  def_macro_noop("\\cohead[]{}")?;
  def_macro_noop("\\ifoot[]{}")?;
  def_macro_noop("\\ofoot[]{}")?;
  def_macro_noop("\\cfoot[]{}")?;
  def_macro_noop("\\lefoot[]{}")?;
  def_macro_noop("\\lofoot[]{}")?;
  def_macro_noop("\\refoot[]{}")?;
  def_macro_noop("\\rofoot[]{}")?;
  def_macro_noop("\\cefoot[]{}")?;
  def_macro_noop("\\cofoot[]{}")?;
  DefMacro!("\\clearscrheadings", "\\ihead{}\\chead{}\\ohead{}\\ifoot{}\\cfoot{}\\ofoot{}");
  DefMacro!("\\clearscrheadfoot", "\\ihead[]{}\\chead[]{}\\ohead[]{}\\ifoot[]{}\\cfoot[]{}\\ofoot[]{}");
  DefMacro!("\\clearscrplain", "\\renewcommand*{\\scrplain@even@left@head}{}\\renewcommand*{\\scrplain@even@middle@head}{}\\renewcommand*{\\scrplain@even@right@head}{}\\renewcommand*{\\scrplain@odd@left@head}{}\\renewcommand*{\\scrplain@odd@middle@head}{}\\renewcommand*{\\scrplain@odd@right@head}{}\\renewcommand*{\\scrplain@even@left@foot}{}\\renewcommand*{\\scrplain@even@middle@foot}{}\\renewcommand*{\\scrplain@even@right@foot}{}\\renewcommand*{\\scrplain@odd@left@foot}{}\\renewcommand*{\\scrplain@odd@middle@foot}{}\\renewcommand*{\\scrplain@odd@right@foot}{}");
});
