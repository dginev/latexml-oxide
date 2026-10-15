//! Shared helper: the shipped class file a class-family binding reads or inspects (`compositio_cls`, `jinst_cls`,
//! `mn2e_cls`).
use crate::prelude::*;

/// The class file in the paper's source that the family binding `binding` interprets: the class the document asked for
/// when the loader answered it with this binding, else the binding's own name; `None` when the source does not have it.
/// The request is the fallback's (`fallback_request`, when its `fallback_target` is this binding: a renamed `JINST_mod`
/// or `myJINST`, 1307.5525, 1407.3938), or the load's own `\@currname` when the dispatch matched the binding by the
/// basename of a path-prefixed request (`JINST-Sample-files/JINST`, whose class is in a subdirectory: 1504.01965,
/// 1410.4420, 1805.09245; the raw load of a basename-keyed package binding resolves its directory the same way,
/// `input_definitions`).
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
/// `jfm_cls`).
pub fn shipped_class_text(binding: &str) -> Result<Option<String>> {
  Ok(requested_shipped_class(binding)?.and_then(|class| {
    find_file(
      &class,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("cls")),
        forbid_ltxml: true,
        ..Default::default()
      }),
    )
    .and_then(|path| std::fs::read(path).ok())
    .map(|bytes| bytes.iter().map(|&b| b as char).collect::<String>())
  }))
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
  let currname = Expand!(Tokens!(T_CS!("\\@currname"))).to_string();
  let fallback_request = lookup_string("fallback_request");
  let request = if !fallback_request.is_empty() && lookup_string("fallback_target") == currname {
    fallback_request
  } else {
    currname
  };
  Ok(!request.is_empty() && require_dependencies_only(&request, ext, ALGORITHM_PACKAGES))
}

/// `text` without its comments (`%` to the end of the line).
fn class_code(text: &str) -> String {
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
