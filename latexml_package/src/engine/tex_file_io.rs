//! TeX File IO
//!
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;
static PSFILE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bpsfile=(.+?)(?:\s|\})").unwrap());

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // File I/O Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
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
    if let Some(path) = find_file(&filename, Some(
      FindFileOptions {forbid_ltxml: true, ..FindFileOptions::default()})) {
      let content_str = LookupString!(&s!("{}_contents",path));
      let content = if content_str.is_empty() {
        None
      } else {
        Some(content_str)
      };
      let mouth = Mouth::create(&path, MouthOptions {
        content,
        .. MouthOptions::default()
      })?;
      AssignValue!(&s!("input_file:{}", port), mouth, Some(Scope::Global));
    }
  });
  DefPrimitive!("\\closein Number", sub[(port)] {
    let file_key = s!("input_file:{}", port);
    let mut finished = false;
    //   close the mouth (if any) and clear the variable
    with_value(&file_key, |mouth_opt|
      if let Some(Stored::Mouth(ref mouth)) = mouth_opt {
        mouth.borrow_mut().finish();
        finished = true;
      });
    if finished {
      AssignValue!(&file_key, false, Some(Scope::Global));
    }
  });

  DefPrimitive!("\\read Number SkipKeyword:to SkipSpaces Token", sub[(port, token)] {
    let mouth_opt =
      if let Some(Stored::Mouth(mouth_stored)) = lookup_value(&format!("input_file:{port}")) {
        Some(mouth_stored)
      } else { None };
    if let Some(mouth_obj) = mouth_opt {
      bgroup();
      AssignValue!("PRESERVE_NEWLINES", 2); // Special EOL/EOF treatment for \read
      AssignValue!("INCLUDE_COMMENTS", false);
      let mut tokens = Vec::new();
      let mut level = 0;
      let mut mouth = mouth_obj.borrow_mut();
      while let Some(t) = mouth.read_token() {
        let cc = t.get_catcode();
        if cc != Catcode::MARKER {
          tokens.push(t);
        }
        match cc {
          Catcode::BEGIN => {level += 1},
          Catcode::END => {level -= 1},
          _ => {}
        };
        if level == 0 && mouth.is_eol() {
          break;
        }
      }
      egroup()?;
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
  DefPrimitive!("\\write Number {}", sub[(port, tokens)] {
    let handle = with_value(&s!("output_file:{}", port), |val_opt|
    if let Some(filename) = val_opt {
       s!("{}_contents",filename)
    } else { String::new() });
    if !handle.is_empty() {
      let mut contents : String = LookupString!(&handle);
      contents.push_str(&Expand!(tokens).untex());
      contents.push('\n');
      AssignValue!(&handle => contents, Some(Scope::Global));
    } else {
      println_stderr!("{}", Expand!(tokens).untex());
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
      // and load LaTeX.pool if not already
      if !lookup_bool("LaTeX.pool_loaded") {
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
      gullet::unread_one(T_CS!("\\ltx@special@graphics"));
    } else {
      Info!("ignored", "special", s!("Unrecognized TeX Special: {arg}"));
    }
  });

  // # adapted from graphicx.sty.ltxml
  // DefKeyVal('SpecialPS', 'angle',   '');
  // DefKeyVal('SpecialPS', 'voffset', '');
  // DefKeyVal('SpecialPS', 'hoffset', '');
  // DefKeyVal('SpecialPS', 'hsize',   '');
  // DefKeyVal('SpecialPS', 'vsize',   '');
  // DefKeyVal('SpecialPS', 'hscale',  '');
  // DefKeyVal('SpecialPS', 'vscale',  '');
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
  // # Since these ultimately generate external resources, it can be useful to have a handle on
  // them. Tag('ltx:graphics', afterOpen => sub { GenerateID(@_, 'g'); });

  //======================================================================
  // output processing
  //----------------------------------------------------------------------
  // \shipout          c  sends the contents of a box to the dvi file.
  // \output           pt holds the token list used to typeset one page.
  DefRegister!("\\output", Tokens!());
});
