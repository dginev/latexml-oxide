//! TeX File IO
//!
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;
static PSFILE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bpsfile=(.+?)(?:\s|\})").unwrap());

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // File I/O Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  // Technically LaTeX, but Package also does file bookkeeping
  DefMacro!(T_CS!("\\@currnamestack"), None, Tokens!());
  Let!("\\@currname", "\\lx@empty");
  Let!("\\@currext", "\\lx@empty");
  // LaTeX2e (2020+) file-hook current-file token lists (latex.ltx:
  // `\tl_new:N \CurrentFile` etc., transiently `\tl_set` during \input).
  // Perl defines them (empty by default); Rust's file-hook layer doesn't,
  // so packages reading them (scrlfile-hook, achemso/koma chains, hyperref
  // driver detection) hit `undefined:\CurrentFile`. Define them as empty
  // token lists — matching Perl's initial/at-rest state (`\detokenize` =
  // empty in both). The actual per-file value isn't tracked here, but an
  // empty defined tl is what these consumers compare against
  // (`\ifx\CurrentFile\CurrentFileUsed`) at the points that matter.
  // Witnesses 2204.03209, 2205.10749, 2311.06870.
  DefMacro!(T_CS!("\\CurrentFile"), None, Tokens!());
  DefMacro!(T_CS!("\\CurrentFilePath"), None, Tokens!());
  DefMacro!(T_CS!("\\CurrentFileUsed"), None, Tokens!());
  DefMacro!(T_CS!("\\CurrentFilePathUsed"), None, Tokens!());
  DefMacro!(
    "\\lx@pushfilename",
    r"\xdef\@currnamestack{{\@currname}{\@currext}{\the\catcode`\@}\@currnamestack}"
  );
  DefMacro!(
    "\\lx@popfilename",
    r"\expandafter\lx@p@pfilename\@currnamestack\@nil"
  );
  DefMacro!(
    "\\lx@p@pfilename {}{}{} Until:\\@nil",
    r"\gdef\@currname{#1}\gdef\@currext{#2}\catcode`\@#3\relax\gdef\@currnamestack{#4}"
  );

  //======================================================================
  // Low-level input
  //----------------------------------------------------------------------
  // \openin           c  opens an auxiliary file for reading.
  // \closein          c  closes an auxiliary file opened for reading.
  // \read             c  reads one or more lines from an auxiliary file.
  // \endinput         c  stops input from a file at the end of the current line.
  // \inputlineno      iq holds the line number of the line last read in the current input file.

  // TeX I/O primitives
  DefPrimitive!("\\openin Number SkipMatch:= SkipSpaces TeXFileName",
  sub[(port, filename)] {
    let port = port.to_string();
    let filename = filename.to_string();
    // possibly should close $port if it's already been opened?
    // Rely on FindFile to enforce any access restrictions
    // Perl: NOT noltxml! \openin is often used to check file existence,
    // and we SHOULD find .ltxml (binding) versions too.
    if let Some(path) = find_file(&filename, None) {
      let content_str = LookupString!(&s!("{}_contents",path));
      let content = if content_str.is_empty() {
        None
      } else {
        Some(content_str)
      };
      // Try to create a Mouth for the file. If it fails (e.g., binding-only
      // file with no disk counterpart), create an empty Mouth so \ifeof
      // returns false (file exists but has no content to read).
      match Mouth::create(&path, MouthOptions {
        content,
        .. MouthOptions::default()
      }) {
        Ok(mouth) => {
          AssignValue!(&s!("input_file:{}", port), mouth, Some(Scope::Global));
        }
        Err(_) => {
          // File was found by find_file (possibly as a binding) but
          // doesn't exist on disk. Create an empty mouth so \ifeof=false.
          let empty_mouth = Mouth::create("literal:", MouthOptions::default())?;
          AssignValue!(&s!("input_file:{}", port), empty_mouth, Some(Scope::Global));
        }
      }
    }
  });
  DefPrimitive!("\\closein Number", sub[(port)] {
    let file_key = s!("input_file:{}", port);
    let mut finished = false;
    //   close the mouth (if any) and clear the variable
    with_value(&file_key, |mouth_opt|
      if let Some(Stored::Mouth(mouth)) = mouth_opt {
        mouth.borrow_mut().finish();
        finished = true;
      });
    if finished {
      AssignValue!(&file_key, false, Some(Scope::Global));
    }
  });

  DefPrimitive!("\\read Number SkipKeyword:to SkipSpaces Token", sub[(port, token)] {
    // Same with_value pattern as etex.rs \readline: Rc::clone the mouth
    // ref instead of cloning the Stored envelope around it.
    let mouth_opt = with_value(&format!("input_file:{port}"), |v| match v {
      Some(Stored::Mouth(mouth)) => Some(Rc::clone(mouth)),
      _ => None,
    });
    if let Some(mouth_obj) = mouth_opt {
      bgroup();
      AssignValue!("PRESERVE_NEWLINES", 2); // Special EOL/EOF treatment for \read
      AssignValue!("INCLUDE_COMMENTS", false);
      let mut tokens = Vec::new();
      let mut level: i32 = 0;
      let mut discard = false;
      let mut mouth = mouth_obj.borrow_mut();
      while let Some(t) = mouth.read_token() {
        let cc = t.get_catcode();
        if cc == Catcode::BEGIN { level += 1; }
        if cc == Catcode::END   { level -= 1; }
        if level < 0            { discard = true; } // silently discard extra } & read till EOL
        if !discard && cc != Catcode::MARKER {
          tokens.push(t);
        }
        if (level == 0 || discard) && mouth.is_eol() {
          break;
        }
      }
      egroup()?;
      if level > 0 {
        Error!("unexpected", "unbalanced",
          s!("Runaway definition? File ended within \\read with unbalanced {{"));
        // Append T_END tokens to balance (non-TeX patch to avoid Fatal)
        for _ in 0..level {
          tokens.push(T_END!());
        }
      }
      DefMacro!(token, None, Tokens::new(tokens), nopack_parameters => true);
    }
  });
  // Note that TeX doesn't actually close the mouth;
  // it just flushes it so that it will close the next time it's read!
  DefMacro!(T_CS!("\\endinput"), None, {
    gullet::flush_mouth();
  });
  DefRegister!("\\inputlineno",Number!(0), readonly => true, getter=> {
    Number::new(gullet::get_locator().from_line as i64)
  });

  //======================================================================
  // Low-level output
  //----------------------------------------------------------------------
  // \openout          c  opens an auxiliary file for writing.
  // \closeout         c  closes an auxiliary file opened for writing.
  // \write            c  writes material to an auxiliary file.
  // \immediate        c  performs the following output command without waiting for \shipout.

  // For output files, we'll write the data to a cached internal copy
  // rather than to the actual file system.
  DefPrimitive!("\\openout Number SkipMatch:= SkipSpaces TeXFileName",
    sub[(port, filename)] {
    let port = port.to_string();
    let filename = filename.to_string();
    let contents_key = &s!("{}_contents",filename);
    AssignValue!(&s!("output_file:{}",port)  => filename,  Some(Scope::Global));
    AssignValue!(contents_key => "",  Some(Scope::Global));
  });

  DefPrimitive!("\\closeout Number", sub[(port)] {
    AssignValue!(&s!("output_file:{}",port), false, Some(Scope::Global));
  });
  // Perl: DefPrimitive('\write Number XGeneralText', sub { … UnTeX($tokens,1) … })
  // XGeneralText is the TeX <general text> — balanced group read with PARTIAL
  // expansion (respects `\the`, `\showthe`, `\unexpanded`, `\detokenize`).
  // Using a raw `{}` followed by `Expand!` over-expands active chars like `~`
  // → `\lx@NBSP`, whose untex leaks the literal string `"\lx@NBSP"` to the
  // aux file; when `\input` reads it back with `@` in OTHER catcode, the CS
  // splits into `\lx`+`@NBSP` — an "undefined \lx" error. arxiv 1112.4846
  // (harvmac `\listrefs`) triggered this.
  DefPrimitive!("\\write Number XGeneralText", sub[(port_n, tokens)] {
    let port = port_n.value_of();
    let handle = with_value(&s!("output_file:{}", port), |val_opt|
    if let Some(filename) = val_opt {
       s!("{}_contents",filename)
    } else { String::new() });
    if !handle.is_empty() {
      let mut contents : String = LookupString!(&handle);
      contents.push_str(&tokens.untex());
      contents.push('\n');
      AssignValue!(&handle => contents, Some(Scope::Global));
    } else if port < 0 {
      NoteLog!(tokens.untex());
    } else {
      Note!(tokens.untex());
    }
  });

  // Since we don't paginate, we're effectively always "shipping out",
  // so all operations are \immediate
  DefPrimitive!("\\immediate", None);

  //======================================================================
  // High-level input
  //----------------------------------------------------------------------
  // \input            c  inserts a file at the current position in the source file.
  DefMacro!("\\input TeXFileName", sub[(name)] {
    let mut tks = name.unlist();
    // If given a LaTeX-style argument, strip braces
    if tks.len() > 1 && tks.first().unwrap().get_catcode() == Catcode::BEGIN
      && tks.last().unwrap().get_catcode() == Catcode::END {
      tks.remove(0);
      tks.pop();
      // and load LaTeX.pool if not already.
      //
      // Skip this auto-load during dump-build (`--init=latex.ltx`).
      // We ARE in the process of dumping LaTeX itself — calling
      // `LoadPool!("LaTeX")` recursively from inside fonttext.ltx's
      // `\input {ot1enc.def}` would re-input latex.ltx, exhaust the
      // gullet, and short-circuit the dump (the cascade observed in
      // Task #28's secondary symptoms). Mirrors Perl iniTeX
      // `mode='Base'`, which never auto-loads LaTeX.pool from
      // `\input` during dump-build.
      if !lookup_bool("LaTeX.pool_loaded")
         && !lookup_bool("INI_TEX_MODE") {
        LoadPool!("LaTeX");
      }
    }
    let reloadable_opts = InputOptions { reloadable: true, ..InputOptions::default() };
    input(&Tokens::new(tks).to_string(), reloadable_opts)?;
  });
  //======================================================================
  // Special output
  //----------------------------------------------------------------------
  // \special          c  sends material to the dvi file for special processing.
  DefPrimitive!("\\special {}", sub[(arg)] {
    let special_str = arg.to_string();
    // recognize one special graphics inclusion case
    if let Some(cap) = PSFILE_REGEX.captures(&special_str) {
      let graphic = cap.get(1).unwrap().as_str();
      RequirePackage!("graphicx", searchpaths_only => true);
      let mut kv = Vec::new();
      for prop in ["voffset","hoffset","hscale","vscale","hsize","vsize","angle"] {
        let prop_regex = Regex::new(&s!("\\b{prop}=(.+?)(?:\\s|\\}})")).unwrap();
        if let Some(cap) = prop_regex.captures(&special_str) {
          let prop_val = cap.get(1).unwrap().as_str();
          if !kv.is_empty() {
            kv.push(T_OTHER!(","));
          }
          kv.push(T_OTHER!(prop));
          kv.push(T_OTHER!("="));
          kv.push(T_OTHER!(prop_val));
        }
      }
      if !kv.is_empty() {
        let mut wrapped = vec![T_OTHER!("[")];
        wrapped.extend(kv);
        wrapped.push(T_OTHER!("]"));
        kv = wrapped;
      }

      gullet::unread_vec(vec![T_BEGIN!(), T_OTHER!(graphic), T_END!()]);
      gullet::unread_vec(kv);
      gullet::unread_one(T_CS!("\\lx@special@graphics"));
    } else {
      Info!("ignored", "special", s!("Unrecognized TeX Special: {arg}"));
    }
  });

  // Adapted from graphicx.sty.ltxml — handles \special{psfile=…} graphics
  DefKeyVal!("SpecialPS", "angle", "");
  DefKeyVal!("SpecialPS", "voffset", "");
  DefKeyVal!("SpecialPS", "hoffset", "");
  DefKeyVal!("SpecialPS", "hsize", "");
  DefKeyVal!("SpecialPS", "vsize", "");
  DefKeyVal!("SpecialPS", "hscale", "");
  DefKeyVal!("SpecialPS", "vscale", "");
  // Perl `tex_file_io.pool.ltxml` (the `\lx@special@graphics`
  // DefConstructor in the commented block below): strip surrounding
  // quotes from the path, search GRAPHICSPATHS for candidates, map
  // psfile-style options (hscale/vscale/hsize/vsize/angle/h+voffset)
  // to graphicx-style options (xscale/yscale/width/height/angle/trim
  // +clip). The prior Rust port was a one-liner emitting
  // `<ltx:graphics graphic='#2'/>` with the raw quoted path and no
  // candidates, so the post-processor couldn't find the source even
  // when it was present in the source directory. Driver:
  // astro-ph0002170 (8 `\special{psfile="figN.ps" angle=… hsize=…}`
  // invocations, all emitting `expected:source` warnings before this
  // fix).
  DefConstructor!(
    "\\lx@special@graphics OptionalKeyVals:SpecialPS Semiverbatim",
    "<ltx:graphics graphic='#path' candidates='#candidates' options='#options'/>",
    properties => sub[args] {
      // arg 0: keyvals, arg 1: graphic path (Semiverbatim)
      let raw = args[1].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      // Perl L276: `$path =~ s/("+)(.+)\g1/$2/;` — strip matched pairs
      // of double-quotes around the value (`\special{psfile="fig.ps"}`
      // arrives with the quotes in the argument string).
      let path_trimmed = raw.trim().trim_matches('"').to_string();
      let candidates = latexml_core::util::image::image_candidates(&path_trimmed);
      // Map psfile-style options to graphicx-style. Perl L282-311.
      let mut options_vec: Vec<String> = Vec::new();
      if let Some(ref kv_digested) = args[0] {
        if let DigestedData::KeyVals(kv) = kv_digested.data() {
          let mut h_off: f64 = 0.0;
          let mut v_off: f64 = 0.0;
          for (key, value) in kv.get_pairs() {
            let s = value.to_string();
            match key.as_str() {
              "hscale" => if let Ok(n) = s.trim().parse::<f64>() {
                options_vec.push(format!("xscale={}", n / 100.0));
              },
              "vscale" => if let Ok(n) = s.trim().parse::<f64>() {
                options_vec.push(format!("yscale={}", n / 100.0));
              },
              "hsize" => options_vec.push(format!("width={}", s.trim())),
              "vsize" => options_vec.push(format!("height={}", s.trim())),
              "angle" => options_vec.push(format!("angle={}", s.trim())),
              "hoffset" => h_off = s.trim().parse::<f64>().unwrap_or(0.0),
              "voffset" => v_off = s.trim().parse::<f64>().unwrap_or(0.0),
              _ => {},
            }
          }
          if h_off != 0.0 || v_off != 0.0 {
            options_vec.push(format!("trim={} {} 0 0", -h_off, -v_off));
            options_vec.push("clip=true".to_string());
          }
        }
      }
      let options = options_vec.join(",");
      Ok(stored_map!("path" => path_trimmed, "candidates" => candidates, "options" => options))
    },
    mode => "text"
  );
  Let!("\\ltx@special@graphics", "\\lx@special@graphics");
  // Original Perl (more complete):
  // DefConstructor('\ltx@special@graphics OptionalKeyVals:SpecialPS Semiverbatim',
  //   "<ltx:graphics graphic='#path' candidates='#candidates' options='#options'/>",
  //   sizer      => \&image_graphicx_sizer,
  //   properties => sub {
  //     my ($stomach, $kv, $path) = @_;
  //     $path = ToString($path); $path =~ s/^\s+//; $path =~ s/\s+$//;
  //     $path =~ s/("+)(.+)\g1/$2/;
  //     my $searchpaths = LookupValue('GRAPHICSPATHS');
  //     my @candidates  = pathname_findall($path, types => ['*'], paths => $searchpaths);
  //     if (my $base = LookupValue('SOURCEDIRECTORY')) {
  //       @candidates = map { pathname_relative($_, $base) } @candidates; }
  //     my $options = '';
  //     if ($kv) {    # remap psfile options to includegraphics options:
  //       if (my $hscale = $kv->getValue('hscale')) {
  //         $hscale = $hscale && int(ToString($hscale)) / 100;
  //         $options .= ',' if $options;
  //         $options .= "xscale=$hscale"; }
  //       if (my $vscale = $kv->getValue('vscale')) {
  //         $vscale = $vscale && int(ToString($vscale)) / 100;
  //         $options .= ',' if $options;
  //         $options .= "yscale=$vscale"; }
  //       if (my $hsize = $kv->getValue('hsize')) {
  //         $hsize = ToString($hsize);
  //         $options .= ',' if $options;
  //         $options .= "width=$hsize"; }
  //       if (my $vsize = $kv->getValue('vsize')) {
  //         $vsize = ToString($vsize);
  //         $options .= ',' if $options;
  //         $options .= "height=$vsize"; }
  //       if (my $angle = $kv->getValue('angle')) {
  //         $angle = ToString($angle);
  //         $options .= ',' if $options;
  //         $options .= "angle=$angle"; }
  //       my $voffset = $kv->getValue('voffset') || 0;
  //       $voffset = $voffset && int(ToString($voffset));
  //       my $hoffset = $kv->getValue('hoffset') || 0;
  //       $hoffset = $hoffset && int(ToString($hoffset));
  //       if ($voffset || $hoffset) {
  //         my $left   = -$hoffset;
  //         my $bottom = -$voffset;
  //         $options .= "," if $options;
  //         $options .= "trim=$left $bottom 0 0,clip=true"; } }
  //     (options => $options, path => $path, candidates => join(',', @candidates)); },
  //   mode => 'text');
  // Since these ultimately generate external resources, it can be useful to have a handle on them.
  Tag!("ltx:graphics", after_open => sub[document, node] {
    document.generate_id(node, "g")?;
  });

  //======================================================================
  // output processing
  //----------------------------------------------------------------------
  // \shipout          c  sends the contents of a box to the dvi file.
  // \output           pt holds the token list used to typeset one page.
  DefRegister!("\\output", Tokens!());
});
