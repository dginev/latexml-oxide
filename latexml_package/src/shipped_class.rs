//! Shared helper: the shipped class file a class-family binding reads or inspects (`compositio_cls`, `jinst_cls`,
//! `mn2e_cls`).
use latexml_core::definition::origin::ProvideOnlyGuard;

use crate::prelude::*;

/// The class file in the paper's source that the family binding `binding` interprets: the class the document asked for
/// when the loader answered it with this binding, else the binding's own name; `None` when the source does not have it.
/// The request is the fallback's (`fallback_request`, when its `fallback_target` is this binding: a renamed `JINST_mod`
/// or `myJINST`, 1307.5525, 1407.3938, and a path-prefixed request loaded as its basename's binding,
/// `JINST-Sample-files/JINST`, whose class is in a subdirectory: 1504.01965, 1410.4420, 1805.09245; `load_class`), or
/// the load's own `\@currname` when it names the binding with a directory (the raw load of a basename-keyed package
/// binding resolves its directory the same way, `input_definitions`).
pub fn requested_shipped_class(binding: &str) -> Result<Option<String>> {
  let is_binding = |name: &str| {
    let name = name.rsplit(['/', '\\']).next().unwrap_or_default();
    let name = strip_cls(name);
    name.eq_ignore_ascii_case(binding)
  };
  let fallback_request = lookup_string("fallback_request");
  let currname = Expand!(Tokens!(T_CS!("\\@currname"))).to_string();
  let class = if !fallback_request.is_empty() && is_binding(&lookup_string("fallback_target")) {
    strip_cls(&fallback_request)
  } else if is_binding(&currname) {
    strip_cls(&currname)
  } else {
    binding
  };
  Ok(
    find_file(
      class,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("cls")),
        forbid_ltxml: true,
        ..Default::default()
      }),
    )
    .map(|_| class.to_string()),
  )
}

/// A stand-in binding's answer when the source lacks the class it reads raw ([`requested_shipped_class`] is `None`):
/// OmniBus, with the loader's missing-file status and `missing_file` Warning for a class it has no binding for
/// (`load_class`; Perl Package.pm:2717-2728), which the binding's presence kept the loader from raising — pdflatex
/// stops on the missing file. Not again for a request the prefix fallback answered with this binding: the loader
/// warned for it. an.cls (astro-ph0611016), cimento, compositio, jinst, the apa family.
pub fn omnibus_without_copy(binding: &str) -> Result<()> {
  let fallback_target = lookup_string("fallback_target");
  let answered = !lookup_string("fallback_request").is_empty()
    && strip_cls(
      fallback_target
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default(),
    )
    .eq_ignore_ascii_case(binding);
  if !answered {
    let name = Expand!(Tokens!(T_CS!("\\@currname"))).to_string();
    let name = if name.is_empty() {
      binding.to_string()
    } else {
      name
    };
    note_status(LogStatus::Missing, Some(&s!("{name}.cls")));
    Warn!(
      "missing_file",
      &name,
      s!("Can't find binding for class {name} (using OmniBus)"),
      "Anticipate undefined macros or environments"
    );
  }
  LoadClass!("OmniBus");
  Ok(())
}

/// The text of the class file [`requested_shipped_class`] finds, its bytes read as Latin-1 characters; `None` when
/// the source does not have it or it cannot be read (so a family binding takes its no-copy choices). A family binding
/// reads the copy the paper ships to follow its version where the versions conflict (`mn2e_cls`, `mdpi_cls`,
/// `jfm_cls`). A copy the document writes with `{filecontents}` is read where the dependency scan reads it, from the
/// virtual files (`require_dependencies_only`).
pub fn shipped_class_text(binding: &str) -> Result<Option<String>> {
  Ok(requested_shipped_class(binding)?.and_then(|class| shipped_file_text(&class, "cls")))
}

/// The text of the paper's copy of the file a stand-in binding answers ([`require_shipped_packages`]'s request: the
/// load's `\@currname`, or the request a versioned fallback answers with this binding), read as
/// [`shipped_class_text`] reads a class; `None` when the source has no copy. The ICML styles read the notice strings
/// a copy edits (`\ICML@appearing`: 2606.28747's "3rd AI for Math Workshop", 2604.14572, 2507.02987, 2506.09099).
pub fn shipped_copy_text(ext: &str) -> Result<Option<String>> {
  let request = stand_in_request()?;
  Ok(if request.is_empty() {
    None
  } else {
    shipped_file_text(&request, ext)
  })
}

/// The text of the file `file` of type `ext` in the paper's source (not a binding), its bytes read as Latin-1
/// characters, or the virtual file a `{filecontents}` wrote.
fn shipped_file_text(file: &str, ext: &str) -> Option<String> {
  find_file(
    file,
    Some(FindFileOptions {
      ext_type: Some(Cow::Owned(ext.to_string())),
      forbid_ltxml: true,
      ..Default::default()
    }),
  )
  .and_then(|path| {
    vfs_read(&path).or_else(|| {
      std::fs::read(path)
        .ok()
        .map(|bytes| bytes.iter().map(|&b| b as char).collect::<String>())
    })
  })
}

/// Does `text` (a class file's) load `package` with `\RequirePackage`/`\usepackage` outside a comment, alone or in a
/// list (`\RequirePackage{graphicx,epstopdf}`), its options spanning lines or not (aa.cls's
/// `\RequirePackage[colorlinks=true,⏎ linkcolor=blue, …]{hyperref}`)? A load inside a conditional or a hook
/// (`\IfFileExists`, `\AtBeginDocument`) counts; `\RequirePackageWithOptions` does not.
pub fn class_text_loads(text: &str, package: &str) -> bool {
  let code = class_code(text);
  ["\\RequirePackage", "\\usepackage"].iter().any(|command| {
    code.match_indices(command).any(|(at, _)| {
      let rest = code[at + command.len()..].trim_start();
      let rest = match rest.strip_prefix('[') {
        Some(options) => options
          .split_once(']')
          .map(|(_, after)| after)
          .unwrap_or_default(),
        None => rest,
      };
      rest
        .trim_start()
        .strip_prefix('{')
        .and_then(|list| list.split_once('}'))
        .is_some_and(|(list, _)| list.split(',').any(|name| name.trim() == package))
    })
  })
}

/// Does `text` (a class file's) contain `needle` outside a comment? (mdpi.cls's `\begin{paracol}` in its
/// `\AfterEndPreamble`, which a copy may have commented out.)
pub fn class_code_contains(text: &str, needle: &str) -> bool { class_code(text).contains(needle) }

/// The astronomical journal abbreviations (`\apj`, `\mnras`, `\aap`…), aas_macros.sty pasted into individual copies of
/// a class whose releases have none: aas_macros' binding provides them where the paper's copy of `binding` defines
/// them, and only the names still undefined (`ProvideOnlyGuard`), so a paper's own definition stays its own.
/// elsarticle (0812.4232 `\def\apj{ApJ}`, 1504.02326, 1807.02064 `\newcommand*\apj{ApJ}`, 2003.05953, 2401.03908
/// `\newcommand\mnras{\ref@jnl{MNRAS}}`), PoS (16 of 92 copies: 0810.0108, 1212.1356, 1708.06808, 1802.02766), svmult
/// (0803.2826, 1801.09940's svmult_aa.cls), svjour3 (0811.4367, 1111.3868) and mdpi (Definitions/mdpi.cls: 2005.05017,
/// 2111.02495; 2510.22959's mdpi_mod.cls).
pub fn provide_shipped_aas_macros(binding: &str) -> Result<()> {
  if shipped_class_text(binding)?.is_some_and(|text| {
    ["\\apj", "\\mnras", "\\aap"]
      .iter()
      .any(|name| class_code_contains(&text, name))
  }) {
    let _provide_only = ProvideOnlyGuard::new();
    RequirePackage!("aas_macros");
  }
  Ok(())
}

/// The algorithm packages, whose choice differs between copies of one class or package: the ICML styles load
/// `algorithm` and `algorithmic` (icml2016.sty:70-71 … icml2025.sty:95-96) and papers edit `algorithmic` out for
/// `algpseudocode` (icml2024.sty:94 "removed due to conflict with algpseudocode", 2402.05403); WileyNJD-v2.cls:3331
/// and WileyNJDv5.cls:6820 load `algorithm, algorithmicx, algpseudocode`, a copy `algorithm2e` (2002.02812); bmvc2k.cls
/// `algorithm` with `algorithmic` (2002.09958:132-133) or with `algpseudocode` (2210.06270:139-142).
pub const ALGORITHM_PACKAGES: &[&str] = &[
  "algorithm",
  "algorithmic",
  "algorithmicx",
  "algpseudocode",
  "algcompatible",
  "algpascal",
  "algc",
  "algmatlab",
  "algorithm2e",
];

/// A binding standing in for the class or package being loaded — as the paper asked for it: the load's
/// `\@currname`, or the request a versioned fallback answers with this binding (`fallback_request` while its
/// `fallback_target` is the binding: `icml2026`, `./icml2018` → icml.sty, 1706.04964) — leaves the algorithm packages
/// to the paper's copy of that file: those of [`ALGORITHM_PACKAGES`] the copy loads are required in its order with its
/// options (Perl's dependency scan, Package.pm:2776-2813, kept to that family: [`require_dependencies_only`]).
/// `false` when the source has no copy, so the binding takes its own default. Algorithmicx's `\State` stays
/// undefined when a stand-in loads `algorithmic`, which bails algorithmicx (algorithmicx.sty.ltxml:9-12), where the
/// paper's copy loads none (icml 1706.03643, 2002.06473, 2106.05074), and `{algorithm}` when it omits the class's own
/// (WileyNJD-v2 1810.03033, 2106.06046; bmvc2k 2210.06270).
pub fn require_shipped_algorithm_packages(ext: &str) -> Result<bool> {
  require_shipped_packages(ext, ALGORITHM_PACKAGES)
}

/// The sub-figure packages, whose choice differs between copies of one class: bmvc2k.cls loads
/// `\RequirePackage{graphicx,subfigure}` (1507.05726's copy:121, 54 of 60 sampled copies), a copy
/// `{graphicx,subcaption}` (2110.13859:133, 1311.3715:117), `\usepackage{caption,subcaption}` (1709.02848:135) or
/// `\usepackage{subfig}` (2107.03463:150) instead; loading one a copy replaced clashes with the paper's own (subfigure
/// and subcaption both define `{subfigure}`).
pub const SUBFIGURE_PACKAGES: &[&str] = &["caption", "subfigure", "subfig", "subcaption"];

/// [`require_shipped_algorithm_packages`] for any such `family` of packages whose choice differs between copies of
/// the file a binding stands in for: those of `family` the paper's copy loads are required in its order with its
/// options; `false` when the source has no copy.
pub fn require_shipped_packages(ext: &str, family: &[&str]) -> Result<bool> {
  let request = stand_in_request()?;
  Ok(!request.is_empty() && require_dependencies_only(&request, ext, family))
}

/// The file a stand-in binding answers, as the paper asked for it: the request a versioned fallback answers with
/// this binding (`fallback_request` while its `fallback_target` is the load's `\@currname`), else `\@currname`.
fn stand_in_request() -> Result<String> {
  let currname = Expand!(Tokens!(T_CS!("\\@currname"))).to_string();
  let fallback_request = lookup_string("fallback_request");
  Ok(
    if !fallback_request.is_empty() && lookup_string("fallback_target") == currname {
      fallback_request
    } else {
      currname
    },
  )
}

/// Does the paper's copy of the class `binding` stands in for load `package` ([`class_text_loads`])? `default` when
/// the source has no (readable) copy — the class as distributed. For a package without a binding, which the
/// dependency scan ([`require_shipped_packages`]) does not load: the binding requires it itself, raw.
pub fn shipped_class_loads_or(binding: &str, package: &str, default: bool) -> Result<bool> {
  Ok(
    shipped_class_text(binding)?
      .map(|text| class_text_loads(&text, package))
      .unwrap_or(default),
  )
}

/// Does the paper's copy of the file a stand-in binding answers ([`shipped_copy_text`]) load `package`
/// ([`class_text_loads`])? `false` when the source has no copy: versions of one class differ (sn-jnl.cls, ws-procs9x6.cls
/// load amsmath in some copies, not in others; `amsmath_sty::require_amsmath_beyond_original`).
pub fn shipped_copy_loads(ext: &str, package: &str) -> Result<bool> {
  Ok(shipped_copy_text(ext)?.is_some_and(|text| class_text_loads(&text, package)))
}

/// Does the paper's own `file` of type `ext` (not a binding) load `package` ([`class_text_loads`])? `false` when the
/// source has none: for a file read by `\input`, which has no `\@currname` of its own ([`shipped_copy_loads`]):
/// Scientific Word 5.x's tcilatex.tex loads amsmath (2212.12000's :851), 2.5's does not (0805.4522's).
pub fn shipped_file_loads(file: &str, ext: &str, package: &str) -> bool {
  shipped_file_text(file, ext).is_some_and(|text| class_text_loads(&text, package))
}

/// Does the document's class option list (`\@classoptionslist`) name `option`? For a class option a binding that
/// does not process its options reads (ieeeaccess's `jtehm`).
pub fn class_option_given(option: &str) -> Result<bool> {
  let Some(defn) = lookup_definition(&T_CS!("\\@classoptionslist"))? else {
    return Ok(false);
  };
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return Ok(false);
  };
  Ok(
    body
      .to_string()
      .split(',')
      .any(|given| given.trim() == option),
  )
}

/// The braced argument of the first `call` in `text` (a class file's) outside a comment that is followed by one, its
/// braces balanced: the default a copy gives a setting command (sn-jnl.cls's `\def\interntxt#1{…}\interntxt{Work done
/// as an intern at Westlake University.}`, 2606.24824's copy :598).
pub fn class_code_argument(text: &str, call: &str) -> Option<String> {
  let code = class_code(text);
  code.match_indices(call).find_map(|(at, _)| {
    let rest = code[at + call.len()..].strip_prefix('{')?;
    let mut depth = 0usize;
    for (i, c) in rest.char_indices() {
      match c {
        '{' => depth += 1,
        '}' if depth == 0 => return Some(rest[..i].to_string()),
        '}' => depth -= 1,
        _ => {},
      }
    }
    None
  })
}

/// `text` without its comments (`%` to the end of the line).
pub fn class_code(text: &str) -> String {
  text
    .lines()
    .map(|line| line.split('%').next().unwrap_or_default())
    .collect::<Vec<_>>()
    .join("\n")
}

/// `name` without a `.cls` extension, in any case.
fn strip_cls(name: &str) -> &str {
  match name.len().checked_sub(4) {
    Some(cut) if name.is_char_boundary(cut) && name[cut..].eq_ignore_ascii_case(".cls") => {
      &name[..cut]
    },
    _ => name,
  }
}

/// A file a binding reads raw — a TeX Live class or package, or an author's class the paper ships — with the binding's
/// adjustments, each placed where the file defines what it adjusts ([`read_raw`]). One declaration per binding, so the
/// read, the no-copy fallback and the order of the adjustments cannot drift between bindings.
pub struct RawRead<'a> {
  /// The file's name, without its extension (`pasa`; a family binding's, chosen as it loads: `compositio`).
  pub name:     &'a str,
  /// Its extension (`cls`, `sty`).
  pub ext:      &'static str,
  /// An author's class, not in TeX Live: read from the paper's copy ([`requested_shipped_class`]); without one the
  /// document gets OmniBus and the missing-file warning, as for any class the paper names and lacks.
  pub shipped:  bool,
  /// `(cs, definition)`: held while the file is read — `definition` (TeX) replaces the meaning of `cs`, locked against
  /// the file's own definition — and the meaning before returns after it (pasa.cls's era `\@popfilename`).
  pub held:     &'static [(&'static str, &'static str)],
  /// Meanings the file overrides with copies of an older kernel's: left to the file while it is read (its own code may
  /// need them), and the meaning before restored after it (pasa.cls's era `\@pushfilename`).
  pub restored: &'static [&'static str],
  /// `(setter, body)`: a store the file defines (`\aaabstract`), redefined after the read to `body`, a frontmatter API
  /// call with `#1` the setter's argument (the body grammar of K11's `STORE_SETTERS`) — only when the file defines it.
  pub setters:  &'static [(&'static str, &'static str)],
  /// TeX run last, for what the declarations above do not cover.
  pub after:    &'static str,
}

impl<'a> RawRead<'a> {
  /// The plain declaration of `name`.`ext`: read raw, nothing held, no setter, nothing after.
  pub const fn new(name: &'a str, ext: &'static str) -> Self {
    RawRead {
      name,
      ext,
      shipped: false,
      held: &[],
      restored: &[],
      setters: &[],
      after: "",
    }
  }
}

/// Read the file `spec` declares, raw, with its adjustments in their order: the held meanings in place and locked, the
/// file, the held and restored meanings back to what they were before it, the setters the file defined redefined, then
/// `after`. `false` when a shipped class is not in the paper's source: the document then has OmniBus
/// ([`omnibus_without_copy`]) and the binding stops there.
pub fn read_raw(spec: &RawRead) -> Result<bool> {
  let file = if spec.shipped {
    let Some(class) = requested_shipped_class(spec.name)? else {
      omnibus_without_copy(spec.name)?;
      return Ok(false);
    };
    class
  } else {
    spec.name.to_string()
  };
  let kept = |cs: &str| s!("\\lx@raw@kept@{}", &cs[1..]);
  for cs in spec.held.iter().map(|(cs, _)| cs).chain(spec.restored) {
    Let!(T_CS!(&kept(cs)), T_CS!(*cs));
  }
  for (cs, definition) in spec.held {
    RawTeX!(*definition);
    AssignValue!(&s!("{cs}:locked") => true, Some(Scope::Global));
  }
  InputDefinitions!(&file, noltxml => true, extension => Some(Cow::Owned(spec.ext.to_string())));
  for (cs, _) in spec.held {
    AssignValue!(&s!("{cs}:locked") => false, Some(Scope::Global));
  }
  for cs in spec.held.iter().map(|(cs, _)| cs).chain(spec.restored) {
    Let!(T_CS!(*cs), T_CS!(&kept(cs)), Scope::Global);
  }
  for (setter, body) in spec.setters {
    if IsDefined!(&T_CS!(*setter)) {
      RawTeX!(&s!("\\long\\def{setter}#1{{{body}}}"));
    }
  }
  if !spec.after.is_empty() {
    RawTeX!(spec.after);
  }
  Ok(true)
}

#[cfg(test)]
mod tests {
  use super::class_text_loads;

  #[test]
  fn class_text_loads_reads_the_load_commands() {
    let class = "\\RequirePackage[T1]{fontenc}\n\\RequirePackage{graphicx,epstopdf}\n%\\RequirePackage{subfig}\n\
                 \\RequirePackageWithOptions{article}\n\\@ifpackageloaded{hyperref}{}{\\RequirePackage[colorlinks=true,\n\
                 urlcolor=blue]{hyperref}}\n\\usepackage{lineno} % \\RequirePackage{caption}\n\\RequirePackage[a]{float}[2001/01/01]";
    for package in [
      "fontenc", "graphicx", "epstopdf", "hyperref", "lineno", "float",
    ] {
      assert!(class_text_loads(class, package), "{package}");
    }
    for package in ["subfig", "article", "caption", "T1", "colorlinks"] {
      assert!(!class_text_loads(class, package), "{package}");
    }
  }
}
