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
  DefMacro!("\\scr@nouppercase", "");
  DefConditional!("\\if@chapter");
  Let!("\\headmark", "\\relax");
  DefMacro!("\\pagemark", "\\thepage");
  DefMacro!("\\defpagestyle OptionalMatch:* {}{}{}", "");
  DefMacro!("\\@defpagestyle[]{}{}{}", "");
  DefMacro!("\\newpagestyle OptionalMatch:* {}{}{}", "");
  DefMacro!("\\@newpagestyle[]{}{}{}", "");
  DefMacro!("\\renewpagestyle OptionalMatch:* {}{}{}", "");
  DefMacro!("\\@renewpagestyle[]{}{}{}", "");
  DefMacro!("\\providepagestyle OptionalMatch:* {}{}{}", "");
  DefMacro!("\\@providepagestyle[]{}{}{}", "");
  DefMacro!("\\deftripstyle OptionalMatch:* {}", "");
  DefMacro!("\\@deftripstyle[]{}", "");
  DefMacro!("\\markleft{}", "");
  DefMacro!("\\automark{}{}", "");
  DefMacro!("\\manualmark", "");
  DefMacro!("\\chapterlevel", "0");
  DefMacro!("\\sectionlevel", "1");
  DefMacro!("\\subsectionlevel", "2");
  DefMacro!("\\subsubsectionlevel", "3");
  DefMacro!("\\paragraphlevel", "4");
  DefMacro!("\\subparagraphlevel", "5");
  DefMacro!("\\settowidthof{}{}", "");
  DefMacro!("\\deftowidthof{}{}", "");
  DefMacro!("\\setheadwidth{}{}", "");
  DefMacro!("\\setfootwidth{}{}", "");
  DefMacro!("\\set@hf@width{}{}{}", "");
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
  DefMacro!("\\scr@setline OptionalMatch:* {}{}", "");
  DefMacro!("\\scrplain@even@left@head", "");
  DefMacro!("\\scrplain@even@middle@head", "");
  DefMacro!("\\scrplain@even@right@head", "");
  DefMacro!("\\scrplain@odd@left@head", "");
  DefMacro!("\\scrplain@odd@middle@head", "");
  DefMacro!("\\scrplain@odd@right@head", "");
  DefMacro!("\\scrplain@even@left@foot", "");
  DefMacro!("\\scrplain@even@middle@foot", "");
  DefMacro!("\\scrplain@even@right@foot", "");
  DefMacro!("\\scrplain@odd@left@foot", "");
  DefMacro!("\\scrplain@odd@middle@foot", "");
  DefMacro!("\\scrplain@odd@right@foot", "");
  DefMacro!("\\ps@scrheadings", "\\let\\ps@plain\\ps@scrplain\\ps@@scrheadings");
  DefMacro!("\\scrheadings@even@left@head", "");
  DefMacro!("\\scrheadings@even@middle@head", "");
  DefMacro!("\\scrheadings@even@right@head", "");
  DefMacro!("\\scrheadings@odd@left@head", "");
  DefMacro!("\\scrheadings@odd@middle@head", "");
  DefMacro!("\\scrheadings@odd@right@head", "");
  DefMacro!("\\scrheadings@even@left@foot", "");
  DefMacro!("\\scrheadings@even@middle@foot", "");
  DefMacro!("\\scrheadings@even@right@foot", "");
  DefMacro!("\\scrheadings@odd@left@foot", "");
  DefMacro!("\\scrheadings@odd@middle@foot", "");
  DefMacro!("\\scrheadings@odd@right@foot", "");
  DefMacro!("\\ihead[]{}", "");
  DefMacro!("\\ohead[]{}", "");
  DefMacro!("\\chead[]{}", "");
  DefMacro!("\\lehead[]{}", "");
  DefMacro!("\\lohead[]{}", "");
  DefMacro!("\\rehead[]{}", "");
  DefMacro!("\\rohead[]{}", "");
  DefMacro!("\\cehead[]{}", "");
  DefMacro!("\\cohead[]{}", "");
  DefMacro!("\\ifoot[]{}", "");
  DefMacro!("\\ofoot[]{}", "");
  DefMacro!("\\cfoot[]{}", "");
  DefMacro!("\\lefoot[]{}", "");
  DefMacro!("\\lofoot[]{}", "");
  DefMacro!("\\refoot[]{}", "");
  DefMacro!("\\rofoot[]{}", "");
  DefMacro!("\\cefoot[]{}", "");
  DefMacro!("\\cofoot[]{}", "");
  DefMacro!("\\clearscrheadings", "\\ihead{}\\chead{}\\ohead{}\\ifoot{}\\cfoot{}\\ofoot{}");
  DefMacro!("\\clearscrheadfoot", "\\ihead[]{}\\chead[]{}\\ohead[]{}\\ifoot[]{}\\cfoot[]{}\\ofoot[]{}");
  DefMacro!("\\clearscrplain", "\\renewcommand*{\\scrplain@even@left@head}{}\\renewcommand*{\\scrplain@even@middle@head}{}\\renewcommand*{\\scrplain@even@right@head}{}\\renewcommand*{\\scrplain@odd@left@head}{}\\renewcommand*{\\scrplain@odd@middle@head}{}\\renewcommand*{\\scrplain@odd@right@head}{}\\renewcommand*{\\scrplain@even@left@foot}{}\\renewcommand*{\\scrplain@even@middle@foot}{}\\renewcommand*{\\scrplain@even@right@foot}{}\\renewcommand*{\\scrplain@odd@left@foot}{}\\renewcommand*{\\scrplain@odd@middle@foot}{}\\renewcommand*{\\scrplain@odd@right@foot}{}");
});
