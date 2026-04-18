use crate::prelude::*;
use std::cmp::Ordering;

LoadDefinitions!({
  // A rough initial draft of the extra commands & registers defined in pdfTeX.

  // See the pdfTeX User's Manual

  // Integer Registers
  DefRegister!("\\pdfoutput"                => Number::new(0));
  DefRegister!("\\pdfminorversion"          => Number::new(4));
  DefRegister!("\\pdfoptionpdfminorversion" => Number::new(4)); // obsolete name
  DefRegister!("\\pdfcompresslevel"         => Number::new(9));
  DefRegister!("\\pdfobjcompresslevel"      => Number::new(0));
  DefRegister!("\\pdfdecimaldigits"         => Number::new(4));
  DefRegister!("\\pdfimageresolution"       => Number::new(72));
  DefRegister!("\\pdfpkresolution"          => Number::new(0));
  DefRegister!("\\pdftracingfonts"          => Number::new(0));
  DefRegister!("\\pdfuniqueresname"         => Number::new(0));
  DefRegister!("\\pdfadjustspacing"         => Number::new(0));
  DefRegister!("\\pdfprotrudechars"         => Number::new(0));
  // \efcode <font> <8bitnumber>  => <integer>
  // \lpfcode <font> <8bitnumber> => <integer>
  // \rpfcode <font> <8bitnumber> => <integer>
  DefRegister!("\\efcode Token Number", Number::new(0));
  DefRegister!("\\lpcode Token Number", Number::new(0));
  DefRegister!("\\rpcode Token Number", Number::new(0));
  DefRegister!("\\knaccode Token Number", Number::new(0));
  DefRegister!("\\knbccode Token Number", Number::new(0));
  DefRegister!("\\knbscode Token Number", Number::new(0));
  DefRegister!("\\shbscode Token Number", Number::new(0));
  DefRegister!("\\stbscode Token Number", Number::new(0));
  DefRegister!("\\tagcode Token Number", Number::new(0));

  DefRegister!("\\pdfforcepagebox"                => Number::new(0));
  DefRegister!("\\pdfoptionalwaysusepdfpagebox"   => Number::new(0));
  DefRegister!("\\pdfinclusionerrorlevel"         => Number::new(0));
  DefRegister!("\\pdfoptionalinclusionerrorlevel" => Number::new(0));
  DefRegister!("\\pdfimagehicolor"                => Number::new(0));
  DefRegister!("\\pdfimageapplygamma"             => Number::new(0));
  DefRegister!("\\pdfgamma"                       => Number::new(0));
  DefRegister!("\\pdfimagegamma"                  => Number::new(0));
  DefRegister!("\\pdfdraftmode"                   => Number::new(0));
  DefRegister!("\\pdfadjustinterwordglue"          => Number::new(0));
  DefRegister!("\\pdfappendkern"                   => Number::new(0));
  DefRegister!("\\pdfgentounicode"                 => Number::new(0));
  DefRegister!("\\pdfinclusioncopyfonts"           => Number::new(0));
  DefRegister!("\\pdfinfoomitdate"                 => Number::new(0));
  DefRegister!("\\pdfpagebox"                      => Number::new(0));
  DefRegister!("\\pdfprependkern"                  => Number::new(0));
  DefRegister!("\\pdfsuppressptexinfo"             => Number::new(0));
  DefRegister!("\\pdfsuppresswarningdupdest"       => Number::new(0));
  DefRegister!("\\pdfsuppresswarningdupmap"        => Number::new(0));
  DefRegister!("\\pdfsuppresswarningpagegroup"     => Number::new(0));

  // Dimen Registers
  DefRegister!("\\pdfhorigin"         => Dimension!("1in"));
  DefRegister!("\\pdfvorigin"         => Dimension!("1in"));
  DefRegister!("\\pdfpagewidth"       => Dimension!("0pt"));
  DefRegister!("\\pdfpageheight"      => Dimension!("0pt"));
  DefRegister!("\\pdflinkmargin"      => Dimension!("0pt"));
  DefRegister!("\\pdfdestmargin"      => Dimension!("0pt"));
  DefRegister!("\\pdfthreadmargin"    => Dimension!("0pt"));
  DefRegister!("\\pdfpxdimen"         => Dimension!("0pt"));
  DefRegister!("\\pdfeachlinedepth"   => Dimension!("0pt"));
  DefRegister!("\\pdfeachlineheight"  => Dimension!("0pt"));
  DefRegister!("\\pdffirstlineheight" => Dimension!("0pt"));
  DefRegister!("\\pdfignoreddimen"    => Dimension!("0pt"));
  DefRegister!("\\pdflastlinedepth"   => Dimension!("0pt"));

  // Token Registers
  DefRegister!("\\pdfpagesattr"     => Tokens!());
  DefRegister!("\\pdfpageattr"      => Tokens!());
  DefRegister!("\\pdfpageresources" => Tokens!());
  DefRegister!("\\pdfpkmode"        => Tokens!());

  // Expandable Commands
  DefMacro!("\\pdftexrevision", "19");
  DefMacro!("\\pdftexbanner", None);
  DefMacro!("\\pdfcreationdate", None);
  DefMacro!("\\pdfpageref Number", None);
  DefMacro!("\\pdfxformname Number", None);
  DefMacro!("\\pdffontname Token", None);
  DefMacro!("\\pdffontobjnum Token", None);
  DefMacro!("\\pdffontsize Token", None);
  DefMacro!("\\pdfincludechars Token {}", None);
  DefMacro!("\\leftmarginkern Number", None);
  DefMacro!("\\rightmarginkern Number", None);
  DefMacro!("\\pdfescapestring {}", None);
  DefMacro!("\\pdfescapename {}", None);
  DefMacro!("\\pdfescapehex {}", None);
  DefMacro!("\\pdfunescapehex {}", None);
  // DefMacro!("\\ifpdfprimitive {}",None);
  // DefMacro!("\\ifpdfabsnum Number"",None);
  // DefMacro!("\\ifpdfabsdim Dimension"",None);
  DefMacro!("\\pdfuniformdeviate Number Token", None);
  DefMacro!("\\pdfnormaldeviate Token", None);
  DefMacro!("\\pdfmdfivesum Number {}", None);
  DefMacro!("\\pdf@mdfivesum Number {}", None);
  DefMacro!("\\pdf@filemdfivesum Number {}", None);
  DefMacro!("\\pdffilesize{}", sub[(file)] {
    // used in expl3's \__file_full_name:n , among others
    let filepath = Expand!(file).to_string();
    if let Some(path) = find_file(&filepath, None) {
      match std::fs::metadata(&path) {
        Ok(meta) => Explode!(meta.len()),
        Err(_) => Vec::new(),
      }
    } else {
      Vec::new() } });
  DefMacro!("\\pdffilemoddate {}", None);
  DefMacro!("\\pdffiledump {}", None);
  // DefMacro(""\pdfcolorstackinit {}",None);

  // Read-only registers
  DefRegister!("\\pdftexversion"           => Number::new(140));
  DefRegister!("\\pdflastobj"              => Number::new(0));
  DefRegister!("\\pdflastxform"            => Number::new(0));
  DefRegister!("\\pdflastximage"           => Number::new(0));
  DefRegister!("\\pdflastximagepages"      => Number::new(0));
  DefRegister!("\\pdflastannot"            => Number::new(0));
  DefRegister!("\\pdflastlink"             => Number::new(0));
  DefRegister!("\\pdflastxpos"             => Number::new(0));
  DefRegister!("\\pdflastypos"             => Number::new(0));
  DefRegister!("\\pdflastdemerits"         => Number::new(0));
  DefRegister!("\\pdfelapsedtime"          => Number::new(0));
  DefRegister!("\\pdfrandomseed"           => Number::new(0));
  DefRegister!("\\pdfshellescape"          => Number::new(0));
  DefRegister!("\\pdflastximagecolordepth" => Number::new(0));
  DefRegister!("\\pdfretval"               => Number::new(0));

  // \pdfximage [ image attr spec ] general text (h, v, m)
  // \pdfrefximage object number (h, v, m)
  // \pdfannot annot type spec (h, v, m)
  // \pdfstartlink [ rule spec ] [ attr spec ] action spec (h, m)
  DefPrimitive!("\\pdfstartlink", None);
  // \pdfendlink (h, m)
  DefPrimitive!("\\pdfendlink", None);
  // \pdfoutline outline spec (h, v, m)
  // \pdfdest dest spec (h, v, m)
  // \pdfthread thread spec (h, v, m)
  // \pdfstartthread thread spec (v, m)
  // \pdfendthread (v, m)
  // \pdfsavepos (h, v, m)

  // See lxRDFa for ideas how this info might be used!
  DefMacro!("\\pdfinfo{}", None);

  // Ugh, what a mess of ugly syntax....
  DefParameterType!(OpenActionSpecification, reader => reader!(_args, _extra, {
    if let Some(_key) = read_keyword(&["openaction"])? {
      if let Some(_action) = read_keyword(&["user", "goto"])? {
        // etc....
      } } }), optional => true);

  // Perl: DefParameterType('OpenAnnotSpecification', sub { ... }, optional, undigested).
  // Reads and discards the pdfTeX annotation-spec prefix:
  //   reserveobjnum  | useobjnum <n>  | stream [attr <text>]
  // then consumes the trailing general-text spec.
  DefParameterType!(OpenAnnotSpecification, reader => reader!(_args, _extra, {
    if read_keyword(&["reserveobjnum"])?.is_some() {
      return Ok(ArgWrap::None);
    } else if read_keyword(&["useobjnum"])?.is_some() {
      let _ = gullet::read_number()?;
    } else if read_keyword(&["stream"])?.is_some()
      && read_keyword(&["attr"])?.is_some() {
        gullet::skip_spaces()?;
        let _ = gullet::read_balanced(ExpansionLevel::Off, false, true)?;
      }
    gullet::skip_spaces()?;
    let _ = gullet::read_balanced(ExpansionLevel::Off, false, true)?;
  }), optional => true);

  // \pdfannot — read annotation spec and discard. Perl pdfTeX.pool L173.
  DefPrimitive!("\\pdfannot OpenAnnotSpecification", None);
  // \pdfobj — same shape. Perl pdfTeX.pool L219.
  DefPrimitive!("\\pdfobj OpenAnnotSpecification", None);

  DefMacro!("\\pdfcatalog{} OpenActionSpecification", "");
  DefMacro!("\\pdfnames{}", "");
  DefMacro!("\\pdftrailer{}", "");
  DefMacro!("\\pdfmapfile{}", "");
  DefMacro!("\\pdfmapline{}", "");
  // \pdffontattr font general text
  // \pdffontexpand font expand spec
  // \vadjust [ pre spec ] filler { vertical mode material } (h, m)
  DefMacro!("\\quitvmode", "");
  // \pdfliteral [ pdfliteral spec ] general text (h, v, m)
  DefPrimitive!(
    "\\pdfliteral OptionalMatch:direct OptionalMatch:page GeneralText",
    None
  );
  // \special pdfspecial spec
  // \pdfresettimer
  DefPrimitive!("\\pdfresettimer", None);
  DefPrimitive!("\\pdfresettimerresettimer", None);
  // \pdfsetrandomseed number
  DefPrimitive!("\\pdfsetrandomseed Number", None);
  // \pdfnoligatures font (really a Token, but at this stub level we
  // just need to consume a single token argument)
  DefPrimitive!("\\pdfnoligatures Token", None);
  // \pdfsavepos — mark current position; no-op stub
  DefPrimitive!("\\pdfsavepos", None);
  // \pdfstartthread / \pdfendthread — thread spec; no-op stubs
  DefPrimitive!("\\pdfstartthread", None);
  DefPrimitive!("\\pdfendthread", None);
  // Per-font extension codes (match \lpcode / \rpcode pattern)
  DefRegister!("\\lpfcode Token Number", Number::new(0));
  DefRegister!("\\rpfcode Token Number", Number::new(0));
  // \pdfprimitive control sequence
  // TODO:
  // https://tex.stackexchange.com/questions/13771/let-a-control-sequence-to-a-redefined-primitive
  DefMacro!("\\pdfprimitive DefToken", "#1"); // we can just ignore the advanced effects for now.

  // \pdfcolorstack stack_num {set|push|pop|current} [general_text]
  //
  // Perl pdfTeX.pool L210: reads stack-number + action keyword, then
  // consumes a trailing general-text spec UNLESS the action was `pop`
  // (which has no spec, just pops the top of the stack). All values
  // are discarded — our engine doesn't emit PDF colorstack operations.
  //
  // Using OptionalMatch for each keyword matches the Perl signature.
  // GeneralText is the balanced-group reader.
  DefPrimitive!(
    "\\pdfcolorstack Number OptionalMatch:set OptionalMatch:push OptionalMatch:pop OptionalMatch:current",
    sub[(_number, _set, _push, pop, _current)] {
      // If action was `pop`, there's no trailing general-text spec.
      // Otherwise read and discard the general-text argument.
      if pop.is_none() {
        gullet::skip_spaces()?;
        let _ = gullet::read_balanced(ExpansionLevel::Off, false, true)?;
      }
    }
  );
  DefMacro!("\\pdfsetmatrix", "");
  DefMacro!("\\pdfsave", "");
  DefMacro!("\\pdfrestore", "");

  // general text → { balanced text }
  // attr spec → attr general text
  // resources spec → resources general text
  // rule spec → ( width | height | depth ) dimension [ rule spec ]
  // object type spec → reserveobjnum |
  // [ useobjnum number ]
  // [ stream [ attr spec ] ] object contents
  // annot type spec → reserveobjnum |
  // [ useobjnum number ] [ rule spec ] general text
  // object contents → file spec | general text
  // xform attr spec → [ attr spec ] [ resources spec ]
  // image attr spec → [ rule spec ] [ attr spec ] [ page spec ] [ colorspace spec ] [ pdf box spec
  // ] outline spec → [ attr spec ] action spec [ count number ] general text
  // action spec → user user-action spec | goto goto-action spec |
  // thread thread-action spec
  // user-action spec → general text
  // goto-action spec → numid |
  // [ file spec ] nameid |
  // [ file spec ] [ page spec ] general text |
  // file spec nameid newwindow spec |
  // file spec [ page spec ] general text newwindow spec
  // thread-action spec → [ file spec ] numid | [ file spec ] nameid
  // open-action spec → openaction action spec
  // colorspace spec → colorspace number
  // pdf box spec → mediabox | cropbox | bleedbox | trimbox | artbox
  // map spec → { [ map modifier ] balanced text }
  // map modifier → + | = | -
  // numid → num number
  // nameid → name general text
  // newwindow spec → newwindow | nonewwindow
  // dest spec → numid dest type | nameid dest type
  // dest type → xyz [ zoom number ] | fitr rule spec |
  // fitbh | fitbv | fitb | fith | fitv | fit
  // thread spec → [ rule spec ] [ attr spec ] id spec
  // id spec → numid | nameid
  // file spec → file general text
  // page spec → page number
  // expand spec → stretch shrink step [ autoexpand ]
  // stretch → number
  // shrink → number
  // step → number
  // pre spec → pre
  // pdfliteral spec → direct | page
  // pdfspecial spec → { [ pdfspecial id [ pdfspecial modifier ] ] balanced text }
  // pdfspecial id → pdf: | PDF:
  // pdfspecial modifier → direct:
  // stack action → set | push | pop | current

  DefMacro!("\\expanded XGeneralText", "#1");

  DefMacro!("\\pdfstrcmp XGeneralText XGeneralText", sub[(first,second)] {
    match first.to_string().cmp(&second.to_string()) {
     Ordering::Greater => Tokens!(T_OTHER!("1")),
     Ordering::Equal => Tokens!(T_OTHER!("0")),
     Ordering::Less => Tokens!(T_OTHER!("-"), T_OTHER!("1"))
    }
  });
  DefMacro!("\\pdfglyphtounicode{}{}", "");
});
