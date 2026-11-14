//! latex-lab-testphase-block.sty — the template types, interfaces, instances and variables the
//! INSTALLED module declares at its top level, read from the file, with inert template code.
//!
//! `\DocumentMetadata` loads the module (latex-lab-testphase-latest.sty:43); raw code then edits or
//! uses its declarations: ltx-talk.cls:1860 `\EditInstance{item}{basic}` (×10 manuals),
//! tagpdfdocu-patches.sty's `{docCommand}` (tagpdf manual, 25 template errors), keytheorems-doc
//! (TL 2025: `theoremblock-0`; on a 2026-06+ format keytheorems.sty:3828-3850: `thmstyle`,
//! `\l__block_thmstyle_tl`, `thm-<style>-1`, `std-list-1`). Its code re-implements list, paragraph
//! and theorem layout with tagging over our constructors (`\@doendpe`, `{itemize}`, `\newtheorem`),
//! so it is not run; each template's code absorbs its keys, `#1` in every version.
//! Guards: `perfect_kernel_batch56::testphase_tagging_sockets_and_block_templates_are_declared`,
//! `perfect_kernel_batchm1::the_block_instances_of_the_file_are_declared`.
use crate::prelude::*;

/// Top-level declarers kept from the file, with their number of braced arguments.
const DECLARERS: &[(&str, usize)] = &[
  ("NewTemplateType", 2),
  ("DeclareTemplateInterface", 4),
  ("DeclareTemplateCode", 4),
  ("DeclareInstance", 4),
  ("DeclareInstanceCopy", 3),
  ("EditInstance", 3),
  ("newcounter", 1),
  ("setcounter", 2),
];

/// The module's user-level commands, inert: each is defined where the file defines it.
const INERT_COMMANDS: &[(&str, &str)] = &[
  ("endblockenv", r"\par"),
  ("BlockEnvEnd", r"\par"),
  ("BlockEnv", r"\UseInstance{blockenv}"),
  (
    "SimpleBlockEnv",
    r"#1#2{\UseInstance{blockenv}{#1}{#2}\BooleanFalse\NoValue\NoValue}",
  ),
];

fn is_name_char(c: u8) -> bool { c.is_ascii_alphabetic() || matches!(c, b'_' | b':' | b'@') }

/// Skips spaces and `%` comments from `i`.
fn skip_blank(s: &[u8], mut i: usize) -> usize {
  while i < s.len() {
    match s[i] {
      b' ' | b'\t' | b'\r' | b'\n' => i += 1,
      b'%' => {
        while i < s.len() && s[i] != b'\n' {
          i += 1;
        }
      },
      _ => break,
    }
  }
  i
}

/// Reads the control sequence at `i` (a `\`): its name and the index past it.
fn read_cs(s: &[u8], i: usize) -> (&[u8], usize) {
  let mut j = i + 1;
  while j < s.len() && is_name_char(s[j]) {
    j += 1;
  }
  if j == i + 1 && j < s.len() {
    j += 1; // a control symbol, `\{` among them
  }
  (&s[i + 1..j], j)
}

/// The index past the balanced group opening at `i`, or `None`.
fn skip_group(s: &[u8], i: usize) -> Option<usize> {
  if s.get(i) != Some(&b'{') {
    return None;
  }
  let (mut depth, mut j) = (0usize, i);
  while j < s.len() {
    match s[j] {
      b'\\' => j = read_cs(s, j).1,
      b'%' => j = skip_blank(s, j),
      b'{' => {
        depth += 1;
        j += 1;
      },
      b'}' => {
        depth -= 1;
        j += 1;
        if depth == 0 {
          return Some(j);
        }
      },
      _ => j += 1,
    }
  }
  None
}

/// The declarations at brace depth 0 of the module's text, as TeX source: the `DECLARERS`
/// statements (a template's code replaced by `\SetKnownTemplateKeys`), the variables it
/// `\<type>_new:N`s (where not defined yet) and the `INERT_COMMANDS` it defines.
fn top_level_declarations(text: &str) -> String {
  let s = text.as_bytes();
  let mut out = String::new();
  let (mut depth, mut i) = (0usize, 0usize);
  while i < s.len() {
    match s[i] {
      b'%' => i = skip_blank(s, i),
      b'{' => {
        depth += 1;
        i += 1;
      },
      b'}' => {
        depth = depth.saturating_sub(1);
        i += 1;
      },
      b'\\' => {
        let (name, next) = read_cs(s, i);
        i = next;
        if depth > 0 {
          continue;
        }
        let name = std::str::from_utf8(name).unwrap_or_default();
        if let Some(&(_, nargs)) = DECLARERS.iter().find(|(n, _)| *n == name) {
          let mut args = Vec::with_capacity(nargs);
          let mut j = i;
          for _ in 0..nargs {
            let start = skip_blank(s, j);
            let Some(end) = skip_group(s, start) else {
              break;
            };
            args.push(&text[start..end]);
            j = end;
          }
          if args.len() < nargs {
            continue;
          }
          out.push('\\');
          out.push_str(name);
          out.push_str(&args.concat());
          if name == "DeclareTemplateCode" {
            // the code itself is skipped; it absorbs the keys of its first argument
            if let Some(end) = skip_group(s, skip_blank(s, j)) {
              j = end;
            }
            let (ty, tmpl) = (args[0], args[1]);
            out.push_str(&format!(r"{{\SetKnownTemplateKeys{ty}{tmpl}{{#1}}}}"));
          }
          out.push('\n');
          i = j;
        } else if name.ends_with("_new:N") {
          let start = skip_blank(s, i);
          if s.get(start) == Some(&b'\\') {
            let (var, end) = read_cs(s, start);
            let var = std::str::from_utf8(var).unwrap_or_default();
            out.push_str(&format!("\\cs_if_exist:NF\\{var}{{\\{name}\\{var}}}\n"));
            i = end;
          }
        } else if name.starts_with("cs_new") && name.ends_with(":Npn") {
          let (cmd, _) = read_cs(s, skip_blank(s, i));
          let cmd = std::str::from_utf8(cmd).unwrap_or_default();
          if let Some((_, body)) = INERT_COMMANDS.iter().find(|(n, _)| *n == cmd) {
            let body = if body.starts_with('#') {
              (*body).to_string()
            } else {
              format!("{{{body}}}")
            };
            out.push_str(&format!("\\cs_new_protected:Npn\\{cmd}{body}\n"));
          }
        }
      },
      _ => i += 1,
    }
  }
  out
}

LoadDefinitions!({
  let text = find_file(
    "latex-lab-testphase-block",
    Some(FindFileOptions {
      ext_type: Some(Cow::Borrowed("sty")),
      ..Default::default()
    }),
  )
  .and_then(|path| std::fs::read_to_string(path).ok())
  .unwrap_or_default();
  let declarations = top_level_declarations(&text);
  raw_tex(&s!("\\ExplSyntaxOn\n{declarations}\\ExplSyntaxOff"))?;
});
