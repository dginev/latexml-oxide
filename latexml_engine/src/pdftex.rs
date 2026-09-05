use std::cmp::Ordering;

use crate::prelude::*;

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

  // \lx@directlua — the LuaTeX escape, evaluated in a persistent external
  // `texlua` (see `lua_bridge.rs`; user directive 2026-08-31: a Lua
  // interpreter may be assumed wherever TeX Live is installed). LuaTeX
  // manual §2.1/§10.3 semantics: the general text is EXPANDED, executed as
  // one chunk in the job-persistent Lua state, and whatever it
  // `tex.print`/`tex.sprint`s is inserted back into the input and read with
  // CURRENT catcodes. A Lua error, or a host without texlua, degrades to an
  // empty expansion with an Info — the content-carrying uses (compute +
  // print) work; node/callback-layer uses are out of scope (bridge docs).
  // Deliberately NOT exposed as `\directlua`: that name's mere EXISTENCE is
  // the LuaTeX-detection probe for babel and friends (`\ifx\directlua
  // \@undefined`), and defining it flipped 26 suite tests onto luatex code
  // paths. Consumers (luacode.sty binding, future opt-ins) use the `\lx@`
  // name; the engine keeps its pdfTeX-model identity.
  DefMacro!("\\lx@directlua XGeneralText", sub[(body)] {
    // XGeneralText already performed the \edef-like PARTIAL expansion while
    // scanning, honoring \noexpand — a second Expand! here re-expanded the
    // no-longer-protected \csname in babel's `[[\noexpand\csname
    // bbl@error\endcsname{]]` idiom, and the resulting macro call ate the
    // Lua text mid-chunk ("unfinished long string", luababel.def L204).
    // Real-engine ground truth (luatex 1.22 probe, 2026-08-31): a \par
    // token in the body (e.g. a blank line inside the chunk) contributes
    // NOTHING to the string Lua receives — `\directlua{ local x = 1 \par
    // @@@ }` errors near '@', not near '\' — while other unexpandable CSes
    // keep their backslash form (`\relax` errors near '\').
    let par = T_CS!("\\par");
    let kept: Vec<Token> = body
      .unlist()
      .into_iter()
      .filter(|t| t != &par)
      .collect();
    let chunk = Tokens::new(kept).to_string();
    match crate::lua_bridge::lua_exec(&chunk) {
      Ok(out) if out.is_empty() => Tokens!(),
      Ok(out) => Tokenize!(TeXString::assembled(out)),
      Err(msg) => {
        // Include the (expanded) chunk head: "not evaluated" without the
        // text we actually sent is undiagnosable — Lua's own [string "…"]
        // excerpt shows the SOURCE author's text, not our expansion of it.
        let head: String = chunk.chars().take(160).collect();
        Info!(
          "lua",
          "directlua",
          s!("\\directlua chunk not evaluated: {msg} | chunk head: {head}")
        );
        Tokens!()
      },
    }
  });
  // \luaescapestring <general text> — escape the text for inclusion inside a
  // Lua string literal (LuaTeX manual: precedes \ " ' and newline with \).
  // XGeneralText already performed the partial expansion while scanning. A second
  // Expand! here over-expanded unexpanded/robust macros inside arguments (e.g.
  // luwa-ul's \unexpanded\expandafter{\underLineKK@test@contents}).
  DefMacro!("\\lx@luaescapestring XGeneralText", sub[(body)] {
    let s = writable_tokens(&body);
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
      match c {
        '\\' | '"' | '\'' => {
          out.push('\\');
          out.push(c);
        },
        '\n' => out.push_str("\\n"),
        _ => out.push(c),
      }
    }
    Tokens!(ExplodeChars!(out))
  });
  // `\primitive` and `\Uchar` are NOT pdfTeX primitives (pdfTeX has only
  // `\pdfprimitive`); `\ifdefined\Uchar` is a Unicode-engine detection probe
  // (ucharcat.sty, math-operator.sty), so both live in the `luatex` profile of
  // latexml.sty. `\Ucharcat` below is the one deliberate exception (expl3's
  // `\char_generate` path, see its note). Guard:
  // `perfect_kernel_batch56::unicode_engine_primitives_stay_undefined_under_pdftex`.
  // \Ucharcat <charcode> <catcode> — XeTeX/LuaTeX Unicode-engine primitive that
  // builds a single char token of the given Unicode scalar + catcode. LaTeXML is
  // Unicode-native, so we provide it (real pdfTeX lacks it). Defining it flips
  // expl3's `\char_generate` (`\if_cs_exist:N \tex_Ucharcat:D`, expl3-code.tex
  // L9210) from the 8-bit `\lowercase{\noexpand~}` active-char trick — which our
  // `\special_relax`/`\noexpand` representation cannot store faithfully (it drops
  // the shadowed char, baking 246 bare `\special_relax` into the dump, e.g.
  // l3text's `\c__text_purify_*` accent tables, and breaking `\codepoint_generate`
  // for combining marks like U+0300) — to the direct charcode+catcode path.
  // Blast radius is tiny: `\Ucharcat` appears only in `\char_generate` across all
  // of expl3 (3 mentions, all there).

  DefMacro!(T_CS!("\\Ucharcat"), None, {
    let charcode = read_number()?.value_of();
    let catcode = read_number()?.value_of();
    match char::from_u32(charcode as u32) {
      Some(ch) => vec![CharToken!(ch, Catcode::from(catcode as u8))],
      None => Vec::new(),
    }
  });

  // Expandable Commands
  DefMacro!("\\pdftexrevision", "19");
  def_macro_noop("\\pdftexbanner")?;
  // pdfTeX manual §8.11: `\pdfcreationdate` expands to the PDF date string
  // `D:YYYYMMDDhhmmss±hh'mm'` of the job start. datetime2.sty:46-48 seeds
  // `\dtm@pdfcreationdate` from it and its `\@dtm@parsepdfdate` splits the
  // fixed-width fields (L1500+); an empty expansion (the old no-op) left
  // `\@dtm@currentminute`… undefined and `\DTMnow` erroring (chemformula /
  // cnltx manuals). Same clock as `\year`/`\time` (SOURCE_DATE_EPOCH honored).
  DefMacro!("\\pdfcreationdate", sub[_args] {
    let year = lookup_register("\\year", Vec::new())?.map_or(0, |v| Number::from(&v).value_of());
    let month = lookup_register("\\month", Vec::new())?.map_or(0, |v| Number::from(&v).value_of());
    let day = lookup_register("\\day", Vec::new())?.map_or(0, |v| Number::from(&v).value_of());
    let time = lookup_register("\\time", Vec::new())?.map_or(0, |v| Number::from(&v).value_of());
    let (hh, mm) = (time / 60, time % 60);
    let stamp = s!("D:{year:04}{month:02}{day:02}{hh:02}{mm:02}00Z");
    Ok(Tokens::new(ExplodeText!(&stamp)))
  });
  def_macro_noop("\\pdfpageref Number")?;
  def_macro_noop("\\pdfxformname Number")?;
  def_macro_noop("\\pdffontname Token")?;
  def_macro_noop("\\pdffontobjnum Token")?;
  def_macro_noop("\\pdffontsize Token")?;
  def_macro_noop("\\pdfincludechars Token {}")?;
  def_macro_noop("\\leftmarginkern Number")?;
  def_macro_noop("\\rightmarginkern Number")?;
  // pdfTeX escape primitives — real implementations, output format verified
  // against live pdfTeX (TL2025, 2026-08-31):
  //   \pdfescapehex{Hello z}   → 48656C6C6F207A        (UPPERCASE hex)
  //   \pdfunescapehex{48656C6C6F} → Hello
  //   \pdfescapestring{a(b)c\ d} → a\(b\)c\\\040d      ((, ), \ backslashed;
  //                                 bytes <33 or >126 as \nnn octal)
  //   \pdfescapename{a b/c#d}  → a#20b#2Fc#23d         (#XX uppercase hex for
  //                                 bytes outside !..~ and PDF delimiters)
  DefMacro!("\\pdfescapehex {}", sub[(arg)] {
    let s = Expand!(arg).to_string();
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
      out.push_str(&format!("{b:02X}"));
    }
    Tokens!(Explode!(out))
  });
  DefMacro!("\\pdfunescapehex {}", sub[(arg)] {
    let s = Expand!(arg).to_string();
    let hex: Vec<u8> = s.bytes().filter(u8::is_ascii_hexdigit).collect();
    let mut out = String::with_capacity(hex.len() / 2);
    for pair in hex.chunks(2) {
      // pdfTeX pads a trailing lone digit with 0 (low nibble).
      let hi = (pair[0] as char).to_digit(16).unwrap_or(0);
      let lo = if pair.len() > 1 {
        (pair[1] as char).to_digit(16).unwrap_or(0)
      } else {
        0
      };
      out.push(char::from((hi * 16 + lo) as u8));
    }
    Tokens!(Explode!(out))
  });
  DefMacro!("\\pdfescapestring {}", sub[(arg)] {
    let s = Expand!(arg).to_string();
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
      match b {
        b'(' | b')' | b'\\' => {
          out.push('\\');
          out.push(char::from(b));
        },
        33..=126 => out.push(char::from(b)),
        _ => out.push_str(&format!("\\{b:03o}")),
      }
    }
    Tokens!(Explode!(out))
  });
  DefMacro!("\\pdfescapename {}", sub[(arg)] {
    let s = Expand!(arg).to_string();
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
      let is_delim = matches!(
        b,
        b'#' | b'/' | b'%' | b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}'
      );
      if (33..=126).contains(&b) && !is_delim {
        out.push(char::from(b));
      } else {
        out.push_str(&format!("#{b:02X}"));
      }
    }
    Tokens!(Explode!(out))
  });
  // DefMacro!("\\ifpdfprimitive {}",None);
  // DefMacro!("\\ifpdfabsnum Number"",None);
  // DefMacro!("\\ifpdfabsdim Dimension"",None);
  // pdfTeX §"random numbers": `\pdfuniformdeviate <number>` expands to a
  // pseudo-random integer in [0, number) (negative bounds mirror), and
  // `\pdfnormaldeviate` to a normal deviate scaled by 2^16, both from a
  // generator seeded by `\pdfsetrandomseed`/`\pdfrandomseed`. Perl
  // (pdfTeX.pool:110-111) makes them empty macros that also EAT the next
  // token — expl3's `\int_rand:nn` (`\tex_uniformdeviate:D 268435456
  // \__fp_sep:`, expl3-code.tex:21876) lost its `\__fp_sep:` and every random
  // integer collapsed to the midpoint, so rejection-sampling loops ("draw
  // until distinct": randintlist-l3 manual, `\randintlist`) never terminate
  // (TokenLimit). A deterministic generator (fixed seed unless the document
  // sets one) keeps conversions reproducible while giving real distributions.
  DefMacro!("\\pdfuniformdeviate Number", sub[(n)] {
    let n = n.value_of();
    let r = pdftex_random_next();
    let v = if n == 0 { 0 } else { (r % n.unsigned_abs()) as i64 * n.signum() };
    Tokens!(Explode!(v.to_string()))
  });
  DefMacro!("\\pdfnormaldeviate", sub[_args] {
    // Box-Muller on two uniforms, scaled like pdfTeX (mean 0, sd 65536).
    let u1 = (pdftex_random_next() % 1_000_000 + 1) as f64 / 1_000_001.0;
    let u2 = (pdftex_random_next() % 1_000_000) as f64 / 1_000_000.0;
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    Tokens!(Explode!(((z * 65536.0) as i64).to_string()))
  });
  // pdfTeX \pdfmdfivesum syntax:
  //   \pdfmdfivesum <general text>      (MD5 of literal string)
  //   \pdfmdfivesum file <general text> (MD5 of file contents)
  // (The Perl port's `Number {}` signature was wrong — there is NO leading
  // number argument.) REAL implementation (inline RFC 1321, unit-tested
  // against the standard vectors): live pdfTeX prints the digest as
  // UPPERCASE hex (`\pdfmdfivesum{abc}` → 900150983CD24FB0D6963F7D28E17F72,
  // verified TL2025 2026-08-31). Witness 2407.02288 (pdfx.sty's
  // `\edef\xmp@docid{\pdfx@mdfivesum{\jobname}}`).
  DefMacro!("\\pdfmdfivesum OptionalMatch:file {}", sub[(file_kw, arg)] {
    let text = Expand!(arg).to_string();
    let digest = if file_kw.is_some() {
      match find_file(&text, None).and_then(|p| std::fs::read(&p).ok()) {
        Some(bytes) => md5_hex_upper(&bytes),
        // pdfTeX yields the empty string for an unreadable file.
        None => String::new(),
      }
    } else {
      md5_hex_upper(text.as_bytes())
    };
    Tokens!(Explode!(digest))
  });
  DefMacro!("\\pdffilesize{}", sub[(file)] {
    // used in expl3's \__file_full_name:n , among others
    let filepath = Expand!(file).to_string();
    if let Some(path) = find_file(&filepath, None) {
      // A `filecontents`-written file is VIRTUAL here (pdfTeX sees a real
      // one): expl3's `\file_full_name:n` existence test is this size, so an
      // `\ior_open:Nn` of such a file found nothing. Guard:
      // `perfect_kernel_batch54::read_consumes_the_physical_line_across_catcode_regimes`.
      if let Some(content) = ::latexml_core::binding::virtual_files::vfs_read(&path) {
        Explode!(content.len())
      } else {
        match std::fs::metadata(&path) {
          Ok(meta) => Explode!(meta.len()),
          Err(_) => Vec::new(),
        }
      }
    } else {
      Vec::new() } });
  // `D:YYYYMMDDhhmmss±hh'mm'` in LOCAL time — verified against live pdfTeX
  // (TL2025 2026-08-31: `D:20260831120047-04'00'`). Empty expansion for a
  // file that does not resolve, matching pdfTeX.
  DefMacro!("\\pdffilemoddate {}", sub[(file)] {
    use chrono::{DateTime, Local};
    let filepath = Expand!(file).to_string();
    let formatted = find_file(&filepath, None)
      .and_then(|p| std::fs::metadata(&p).ok())
      .and_then(|m| m.modified().ok())
      .map(|t| {
        let dt: DateTime<Local> = t.into();
        // %z gives ±hhmm; pdfTeX writes ±hh'mm'
        let z = dt.format("%z").to_string();
        let (zh, zm) = z.split_at(3);
        format!("{}{}'{}'", dt.format("D:%Y%m%d%H%M%S"), zh, zm)
      })
      .unwrap_or_default();
    Tokens!(Explode!(formatted))
  });
  def_macro_noop("\\pdffiledump {}")?;
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
  // Real pdfTeX reads optional `[image attr spec]` then a balanced text
  // (the file path). Stub: drop a leading `[...]` if present, then
  // consume one balanced general-text arg. No PDF emission. Driver:
  // 2406.14142 (`\pdfximage{...}` in graphics-bbox-precompute path).
  DefPrimitive!("\\pdfximage", sub[_args] {
    skip_spaces()?;
    if if_next(T_OTHER!("["))? {
      // discard up to matching `]`
      while let Some(t) = read_token()? {
        if matches!(t.get_catcode(), Catcode::OTHER) && t.to_string() == "]" {
          break;
        }
      }
    }
    skip_spaces()?;
    let _ = read_balanced(ExpansionLevel::Off, false, true)?;
    Ok(vec![])
  });
  // \pdfrefximage object number (h, v, m) — discard the object number
  def_primitive_noop("\\pdfrefximage Number")?;
  // \pdfrefobj object_number / \pdfrefxform xform_number — discard the
  // number; no PDF output. pdfTeX-only primitives invoked by some
  // packages that declare-then-reference pdf objects (e.g. zref-savepos
  // path on certain papers). Witness cluster: arXiv:2506.21632 / .08091.
  def_primitive_noop("\\pdfrefobj Number")?;
  def_primitive_noop("\\pdfrefxform Number")?;
  // \pdfannot annot type spec (h, v, m)
  // \pdfstartlink [ rule spec ] [ attr spec ] action spec (h, m)
  def_primitive_noop("\\pdfstartlink")?;
  // \pdfendlink (h, m)
  def_primitive_noop("\\pdfendlink")?;
  // \pdfoutline outline spec (h, v, m) — `[attr spec] action spec [count
  // number] general text` (pdfTeX manual §8.13); `\pdfdest dest spec` is
  // `name/num <spec> <dest kind>` (§8.14; kinds `xyz [zoom N]`, `fit`,
  // `fith`, `fitv`, `fitb`, `fitbh`, `fitbv`, `fitr <rule spec>`). Both
  // produce PDF navigation only, but the spec MUST be consumed — a bare
  // no-op leaked `attr`/`user` into the text (tools-overview.tex:93
  // `\pdfoutline attr {…} user {…} {[#1]}`; Perl pdfTeX.pool:179-180 only
  // comments them, KPE #162). Guards:
  // `perfect_kernel_batch54::pdfoutline_and_pdfdest_consume_their_specs`.
  DefParameterType!(OutlineSpecification, reader => reader!(_args, _extra, {
    if read_keyword(&["attr"])?.is_some() {
      skip_spaces()?;
      let _ = read_balanced(ExpansionLevel::Off, false, true)?;
    }
    read_action_spec()?;
    if read_keyword(&["count"])?.is_some() {
      let _ = read_number()?;
    }
    skip_spaces()?;
    let _ = read_balanced(ExpansionLevel::Off, false, true)?;
  }), optional => true);
  def_primitive_noop("\\pdfoutline OutlineSpecification")?;
  DefParameterType!(DestSpecification, reader => reader!(_args, _extra, {
    if read_keyword(&["num"])?.is_some() {
      let _ = read_number()?;
    } else if read_keyword(&["name"])?.is_some() {
      skip_spaces()?;
      let _ = read_balanced(ExpansionLevel::Off, false, true)?;
    }
    if read_keyword(&["xyz"])?.is_some() {
      if read_keyword(&["zoom"])?.is_some() {
        let _ = read_number()?;
      }
    } else if read_keyword(&["fitr"])?.is_some() {
      while read_keyword(&["width", "height", "depth"])?.is_some() {
        let _ = read_dimension()?;
      }
    } else {
      let _ = read_keyword(&["fitbh", "fitbv", "fitb", "fith", "fitv", "fit"])?;
    }
  }), optional => true);
  def_primitive_noop("\\pdfdest DestSpecification")?;
  // \pdfthread thread spec (h, v, m)
  // \pdfstartthread thread spec (v, m)
  // \pdfendthread (v, m)
  // \pdfsavepos (h, v, m)

  // See lxRDFa for ideas how this info might be used!
  def_macro_noop("\\pdfinfo{}")?;

  // Ugh, what a mess of ugly syntax....
  DefParameterType!(OpenActionSpecification, reader => reader!(_args, _extra, {
    if let Some(_key) = read_keyword(&["openaction"])?
      && let Some(_action) = read_keyword(&["user", "goto"])? {
        // etc....
      } }), optional => true);

  // Perl: DefParameterType('OpenAnnotSpecification', sub { ... }, optional, undigested).
  // Reads and discards the pdfTeX annotation-spec prefix:
  //   reserveobjnum  | useobjnum <n>  | stream [attr <text>]
  // then the `annot type spec`'s optional `rule spec` — `(width|height|depth)
  // dimension [rule spec]`, the same loop as `RuleSpecification` (tex_box.rs)
  // — which Perl pdfTeX.pool:156-171 omits: pdfmarginpar.sty:142
  // `\expandafter\pdfannot\pdfmarginpar@rulespec{…}` with a `width=`/`height=`
  // key passes `width 4cm height 0.5cm {…}` and the reader met `w` where it
  // wanted `{` ("Expected opening '{'", pdfmarginpar doc; KNOWN_PERL_ERRORS
  // #151). Then consumes the trailing general-text spec.
  DefParameterType!(OpenAnnotSpecification, reader => reader!(_args, _extra, {
    if read_keyword(&["reserveobjnum"])?.is_some() {
      return Ok(ArgWrap::None);
    } else if read_keyword(&["useobjnum"])?.is_some() {
      let _ = read_number()?;
    } else if read_keyword(&["stream"])?.is_some()
      && read_keyword(&["attr"])?.is_some() {
        skip_spaces()?;
        let _ = read_balanced(ExpansionLevel::Off, false, true)?;
      }
    while read_keyword(&["width", "height", "depth"])?.is_some() {
      let _ = read_dimension()?;
    }
    skip_spaces()?;
    let _ = read_balanced(ExpansionLevel::Off, false, true)?;
  }), optional => true);

  // \pdfannot — read annotation spec and discard. Perl pdfTeX.pool L173.
  def_primitive_noop("\\pdfannot OpenAnnotSpecification")?;
  // \pdfobj — same shape. Perl pdfTeX.pool L219.
  def_primitive_noop("\\pdfobj OpenAnnotSpecification")?;

  def_macro_noop("\\pdfcatalog{} OpenActionSpecification")?;
  def_macro_noop("\\pdfnames{}")?;
  def_macro_noop("\\pdftrailer{}")?;
  // \pdftrailerid{<id>} — pdfTeX primitive that overrides the PDF
  // trailer ID. Used by `anonymous-review` style preamble redaction
  // (e.g. `\pdftrailerid{redacted}`). No-op for HTML/XML output.
  // Witness 2403.06807.
  def_macro_noop("\\pdftrailerid{}")?;
  def_macro_noop("\\pdfmapfile{}")?;
  def_macro_noop("\\pdfmapline{}")?;
  // \pdffontattr font general text
  // \pdffontexpand font expand spec
  // \vadjust [ pre spec ] filler { vertical mode material } (h, m)
  def_macro_noop("\\quitvmode")?;
  // \pdfliteral [ pdfliteral spec ] general text (h, v, m)
  DefPrimitive!(
    "\\pdfliteral OptionalMatch:direct OptionalMatch:page GeneralText",
    None
  );
  // \special pdfspecial spec
  // \pdfresettimer
  def_primitive_noop("\\pdfresettimer")?;
  def_primitive_noop("\\pdfresettimerresettimer")?;
  // \pdfsetrandomseed number
  DefPrimitive!("\\pdfsetrandomseed Number", sub[(seed)] {
    let seed = seed.value_of();
    assign_register("\\pdfrandomseed", RegisterValue::Number(Number::new(seed)), Some(Scope::Global), Vec::new())?;
    pdftex_random_seed(seed);
  });
  // \pdfnoligatures font (really a Token, but at this stub level we
  // just need to consume a single token argument)
  def_primitive_noop("\\pdfnoligatures Token")?;
  // \pdfsavepos — saves current (x, y) page position into
  // \pdflastxpos / \pdflastypos. Stub as no-op; the position is never
  // actually computed in our XML output so the saved values stay 0.
  // zref-savepos.sty L57-63 PackageErrors out if \pdfsavepos is
  // undefined ("not supported"); making it defined lets zref-savepos
  // proceed normally. linegoal.sty's gated code uses \globcount /
  // \globdimen — both of which are now defined in etex.rs (L545/547)
  // so the linegoal cascade is no longer a concern.
  // Witnesses (zref-savepos): 2503.15628, 2503.18497, 2504.03449,
  // 2504.03565, 2504.05447, 2504.05890.
  def_primitive_noop("\\pdfsavepos")?;
  // \pdfstartthread / \pdfendthread — thread spec; no-op stubs
  def_primitive_noop("\\pdfstartthread")?;
  def_primitive_noop("\\pdfendthread")?;
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
        skip_spaces()?;
        let _ = read_balanced(ExpansionLevel::Off, false, true)?;
      }
    }
  );
  def_macro_noop("\\pdfsetmatrix")?;
  def_macro_noop("\\pdfsave")?;
  def_macro_noop("\\pdfrestore")?;

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
  def_macro_noop("\\pdfglyphtounicode{}{}")?;

  // LuaTeX integer parameter (LuaTeX manual §2.5: hyphenation behaviour for
  // words with explicit hyphens/automatic discretionaries; a line-breaking
  // knob with no XML-content meaning). LuaLaTeX-authored manuals set it in
  // their preamble (nicematrix.tex L303).
  DefRegister!("\\automatichyphenmode" => Number::new(0));
});

/// MD5 (RFC 1321), digest as UPPERCASE hex — the format pdfTeX's
/// `\pdfmdfivesum` prints (verified against live pdfTeX TL2025). Inline
/// implementation: 64-entry sine table generated exactly as the RFC defines
/// it (`floor(2^32 · |sin(i+1)|)`), guarded below by the RFC's own test
/// vectors.
/// Consume a pdfTeX `action spec` (pdfTeX manual §8.11, and the grammar
/// quoted above): `user <general text>` | `goto <goto-action spec>` |
/// `thread <thread-action spec>`. The goto form is `[file <text>] num N` |
/// `[file <text>] name <text> [newwindow|nonewwindow]` | `[file <text>]
/// [page N] <general text> [newwindow|nonewwindow]`. Nothing is kept — the
/// actions are PDF navigation — but every token of the spec must be eaten
/// so it does not fall into the text. Shared by `\pdfoutline` and
/// `\pdfstartlink`-style readers.
fn read_action_spec() -> Result<()> {
  let Some(kind) = read_keyword(&["user", "goto", "thread"])? else {
    return Ok(());
  };
  skip_spaces()?;
  if kind == "user" {
    let _ = read_balanced(ExpansionLevel::Off, false, true)?;
    return Ok(());
  }
  if read_keyword(&["file"])?.is_some() {
    skip_spaces()?;
    let _ = read_balanced(ExpansionLevel::Off, false, true)?;
  }
  if read_keyword(&["num"])?.is_some() {
    let _ = read_number()?;
  } else if read_keyword(&["name"])?.is_some() {
    skip_spaces()?;
    let _ = read_balanced(ExpansionLevel::Off, false, true)?;
  } else if kind == "goto" {
    if read_keyword(&["page"])?.is_some() {
      let _ = read_number()?;
    }
    skip_spaces()?;
    let _ = read_balanced(ExpansionLevel::Off, false, true)?;
  }
  let _ = read_keyword(&["newwindow", "nonewwindow"])?;
  Ok(())
}

fn md5_hex_upper(data: &[u8]) -> String {
  const S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, //
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, //
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, //
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
  ];
  let mut k = [0u32; 64];
  for (i, ki) in k.iter_mut().enumerate() {
    *ki = (((i as f64) + 1.0).sin().abs() * 4294967296.0) as u32;
  }
  let (mut a0, mut b0, mut c0, mut d0) =
    (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);
  let mut m = data.to_vec();
  let bitlen = (data.len() as u64).wrapping_mul(8);
  m.push(0x80);
  while m.len() % 64 != 56 {
    m.push(0);
  }
  m.extend_from_slice(&bitlen.to_le_bytes());
  for chunk in m.as_chunks::<64>().0 {
    let mut w = [0u32; 16];
    for (j, wj) in w.iter_mut().enumerate() {
      *wj = u32::from_le_bytes(chunk[4 * j..4 * j + 4].try_into().unwrap());
    }
    let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
    for i in 0..64 {
      let (f, g) = match i / 16 {
        0 => ((b & c) | (!b & d), i),
        1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
        2 => (b ^ c ^ d, (3 * i + 5) % 16),
        _ => (c ^ (b | !d), (7 * i) % 16),
      };
      let tmp = d;
      d = c;
      c = b;
      b = b.wrapping_add(
        a.wrapping_add(f)
          .wrapping_add(k[i])
          .wrapping_add(w[g])
          .rotate_left(S[i]),
      );
      a = tmp;
    }
    a0 = a0.wrapping_add(a);
    b0 = b0.wrapping_add(b);
    c0 = c0.wrapping_add(c);
    d0 = d0.wrapping_add(d);
  }
  let mut out = String::with_capacity(32);
  for word in [a0, b0, c0, d0] {
    for byte in word.to_le_bytes() {
      out.push_str(&format!("{byte:02X}"));
    }
  }
  out
}

#[cfg(test)]
mod md5_tests {
  use super::md5_hex_upper;

  /// RFC 1321 appendix A.5 test vectors (uppercased to pdfTeX's format).
  #[test]
  fn rfc1321_vectors() {
    assert_eq!(md5_hex_upper(b""), "D41D8CD98F00B204E9800998ECF8427E");
    assert_eq!(md5_hex_upper(b"abc"), "900150983CD24FB0D6963F7D28E17F72");
    assert_eq!(
      md5_hex_upper(b"message digest"),
      "F96B697D7CB7938D525A2F31AAF161D0"
    );
    // A >64-byte message exercises the multi-chunk path.
    assert_eq!(
      md5_hex_upper(
        b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"
      ),
      "57EDF4A22BE3C955AC49DA2E2107B67A"
    );
  }
}

thread_local! {
  /// pdfTeX random-number state (xorshift64*), reset per conversion by the
  /// engine's thread-state reset through [`pdftex_random_seed`].
  static PDFTEX_RANDOM: std::cell::Cell<u64> =
    const { std::cell::Cell::new(0x9E37_79B9_7F4A_7C15) };
}

/// Re-seed the pdfTeX generator (`\pdfsetrandomseed`; 0 selects the default
/// fixed seed so an unseeded document converts reproducibly).
pub fn pdftex_random_seed(seed: i64) {
  let s = if seed == 0 {
    0x9E37_79B9_7F4A_7C15
  } else {
    (seed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1
  };
  PDFTEX_RANDOM.with(|c| c.set(s));
}

/// Next 63-bit pseudo-random value (xorshift64*).
fn pdftex_random_next() -> u64 {
  PDFTEX_RANDOM.with(|c| {
    let mut x = c.get();
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    c.set(x);
    x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 1
  })
}
