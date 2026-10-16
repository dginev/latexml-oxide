//! Shared helper: the shipped class file a class-family binding reads or inspects (`compositio_cls`, `jinst_cls`,
//! `mn2e_cls`).
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
