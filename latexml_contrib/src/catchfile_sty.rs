//! catchfile.sty (H. Oberdiek) — `\CatchFileDef` / `\CatchFileEdef`.
//!
//! catchfile.sty:251-301: `\CatchFileDef\cs{file}{setup}` opens a group,
//! runs `setup` (catcode changes, `\endlinechar`), reads the whole file's
//! tokens under those catcodes, closes the group and defines `\cs` with them
//! at the outer level — unexpanded for `\CatchFileDef`, `\xdef`-expanded for
//! `\CatchFileEdef` (L251-261). A missing file defines `\cs` empty and is a
//! package error (L240-245).
//!
//! The setup argument is what makes the read faithful: codehigh's
//! `\dochighinput` reads a `.sty` with `\catcode`\#=12` so parameter
//! characters survive as text (fontscale-code, cistercian manuals), makron.sty
//! L61 reads `\jobname.runs` for a counter (arXiv 1611.01359), mnras tables
//! (arXiv 2210.08043). Guards:
//! `perfect_kernel_batch54::catchfiledef_reads_under_setup_catcodes_and_edef_expands`,
//! `binding_singletons_56::catchfile_expands_the_name_and_reads_filecontents`.
//!
//! Both keep catchfile's own protocol, because the setup may replace its
//! parts: catchfilebetweentags.sty `\CatchFBT@Work` redefines `\CatchFile@Do`
//! (the reader of the file's tokens, delimited by `\CatchFile@EOF`) and
//! `\everyeof` to capture only the text between two tags
//! (factura-ejemplo-prefactura: `undefined:\CatchFile@EOF`). Only the file
//! opening is native: `\lx@catchfile@input` opens the file as an `\input`
//! would (a mouth whose end inserts `\everyeof`), without `\input`'s binding
//! lookup — a `.sty` is caught as text, never loaded. Guard:
//! `binding_singletons_56::catchfilebetweentags_uses_the_eof_protocol`.
use latexml_core::mouth::{Mouth, MouthOptions};
use latexml_package::prelude::*;

/// catchfile's `\CatchFile@Input` (catchfile.sty:175-183: the `\input`
/// primitive) applied to `\CatchFile@File`: the found file becomes the next
/// input level, read under the catcodes in force, and its end inserts
/// `\everyeof` (as content.rs `load_tex_content` opens an `\input` file).
///
/// `\input` scans a file name (tex.web §526). The name is complete here (it is
/// `\CatchFile@File`), so the scan ends at the next token: a space — the
/// `\space` of `\CatchFileEdef` (:259) — is consumed, anything else — the
/// `\relax` of `\CatchFileDef` (:299) — is backed up to follow the file.
///
/// The file's end is opaque to a balanced read ([`BalancedBoundary::Opaque`]):
/// a scan that meets it is a runaway, as TeX's is (tex.web §338), so an
/// unbalanced brace in the file cannot swallow the document after it. Only
/// `\noexpand` reads past it (tex.web §367, `read_token_across_input_ends`):
/// that is how `\CatchFileEdef`'s `\everyeof{\noexpand}` (:257) carries its
/// `\xdef` into the `}` after the file. A disk file is read as a file mouth, so
/// its bytes are decoded under the input encoding and checked for binary
/// content. Guard: `noexpand_input_ends::catchfile_edef_reads_the_file_bytes`
/// (Latin-1 `café`).
fn open_caught_file(file: Tokens) -> Result<()> {
  let path = do_expand(file)?.to_string();
  if let Some(next) = read_x_token(Some(false), false, Some(true))?
    && next.get_catcode() != Catcode::SPACE
  {
    unread_one(next);
  }
  open_mouth_with(
    Mouth::create(&path, MouthOptions {
      content: vfs_read(&path),
      ..MouthOptions::default()
    })?,
    true,
    BalancedBoundary::Opaque,
  );
  mark_everyeof_mouth();
  Ok(())
}

LoadDefinitions!({
  // catchfile.sty:157: infwarerr's `\@PackageError`, which
  // `\CatchFile@NotFound` raises.
  RequirePackage!("infwarerr");
  // catchfile.sty:224-238 (the `\IfFileExists` branch): `\CatchFile@File` is
  // the found file, or `\relax`. The name is built as the kernel's
  // `\IfFileExists` builds it (sect13.rs): expanded inside a `\csname`, as
  // `\set@curr@file` does, then file-substituted — so `\jobname.runs` names
  // the job's file (makron.sty:61, arXiv 1611.01359).
  DefPrimitive!("\\CatchFile@CheckFileExists{}", sub[(path)] {
    let name = expand_as_csname_text(path)?.to_string();
    let name = substitute_file_request(&name).unwrap_or(name);
    match find_file(&name, None) {
      Some(found) => {
        def_macro(T_CS!("\\CatchFile@File"), None, Tokens::new(ExplodeText!(found)), None)?;
      },
      None => let_i(&T_CS!("\\CatchFile@File"), &T_CS!("\\relax"), None),
    }
    Ok(())
  });
  // catchfile.sty:240-245.
  RawTeX!(
    r"\def\CatchFile@NotFound#1#2{%
  \def#1{}%
  \@PackageError{catchfile}{%
    File `#2' not found%
  }\@ehc
}"
  );
  DefMacro!("\\lx@catchfile@input{}", sub[(file)] {
    open_caught_file(file)?;
    Ok(Tokens!())
  });
  // catchfile.sty:251-261, `\CatchFile@Input` being `\lx@catchfile@input`.
  RawTeX!(
    r"\long\def\CatchFileEdef#1#2#3{%
  \CatchFile@CheckFileExists{#2}%
  \ifx\CatchFile@File\relax
    \CatchFile@NotFound{#1}{#2}%
  \else
    \begingroup
      \everyeof{\noexpand}%
      #3%
      \xdef\CatchFile@Contents{\lx@catchfile@input\CatchFile@File\space}%
    \endgroup
    \let#1\CatchFile@Contents
  \fi}"
  );
  // catchfile.sty:264-301 (the e-TeX branch), `\CatchFile@Input` being
  // `\lx@catchfile@input`.
  RawTeX!(
    r"\long\def\CatchFileDef#1#2#3{%
  \CatchFile@CheckFileExists{#2}%
  \ifx\CatchFile@File\relax
    \CatchFile@NotFound{#1}{#2}%
  \else
    \begingroup
      \everyeof\expandafter{\CatchFile@EOF\expandafter\CatchFile@Finish\noexpand}%
      \expandafter\long\expandafter\def\expandafter\CatchFile@Do
          \expandafter##\expandafter1\CatchFile@EOF{%
        \edef\CatchFile@Finish{\endgroup\unexpanded{\edef#1{\unexpanded{##1}}}}}%
      #3\relax
    \expandafter\CatchFile@Do\lx@catchfile@input\CatchFile@File\relax
  \fi}"
  );
  // catchfile.sty:302-309: the delimiter is `@@` with catcodes 8 and 3, which
  // no file text can contain.
  RawTeX!(
    r"\begingroup\lccode65=64 \lccode66=64 \catcode65=8 \catcode66=3
\lowercase{\endgroup\def\CatchFile@EOF{AB}}"
  );
});
