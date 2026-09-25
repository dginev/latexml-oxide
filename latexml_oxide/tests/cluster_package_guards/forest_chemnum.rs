//! forest node keys, `\Forest*` and libraries; chemnum declaration targets
//! (perfect-kernel round 12, P6-P8).
use std::process::Command;

use latexml::util::test::{assert_element, rng_error_count};

use super::perfect_kernel_batch46::{convert, convert_args, error_count, warning_count};

/// P6: a node's options are a pgfkeys keylist (forest.sty:1423-1426 `new
/// node` → `content=<spec>`; pgfkeys.code.tex:357, 369, 507-520). `edge
/// label` becomes the node's `ltx_forest_edge_label` text, `tier` and
/// `phantom` classes (a phantom node draws neither its label nor the edge
/// label, forest.sty:7628-7651), `name` its `xml:id`; `for tree` and the
/// `\forestset` `default preamble` reach every node before its own options
/// (the tree preamble's `e` overrides the default preamble's `dp`, a node's
/// own edge label overrides both), and a `\forestset` style (`word`)
/// expands where it is used.
#[test]
fn forest_node_keys_become_structure() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/forest_node_keys_structure.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "inline-block",
    &[r#"class="ltx_forest_tree""#],
    r#"<inline-block class="ltx_forest_tree">
      <inline-enumerate class="ltx_forest">
        <inline-item class="ltx_forest_node">
          <text class="ltx_forest_node_content">VP</text>
          <inline-enumerate class="ltx_forest_children">
            <inline-item class="ltx_forest_node">
              <text class="ltx_forest_edge_label">spec</text>
              <text class="ltx_forest_node_content">V</text>
            </inline-item>
            <inline-item class="ltx_forest_node ltx_forest_tier_word">
              <text class="ltx_forest_edge_label">e</text>
              <text class="ltx_forest_node_content">DP</text>
              <inline-enumerate class="ltx_forest_children">
                <inline-item class="ltx_forest_node" xml:id="forest.dnode">
                  <text class="ltx_forest_edge_label">head</text>
                  <text class="ltx_forest_node_content">D</text>
                </inline-item>
              </inline-enumerate>
            </inline-item>
            <inline-item class="ltx_forest_node ltx_forest_phantom"/>
            <inline-item class="ltx_forest_node ltx_forest_tier_word">
              <text class="ltx_forest_edge_label">e</text>
              <text class="ltx_forest_node_content">N</text>
            </inline-item>
          </inline-enumerate>
        </inline-item>
      </inline-enumerate>
    </inline-block>"#,
  );
  // The second tree has no preamble of its own: the default preamble's
  // `for tree` edge label reaches B (the root has no edge).
  let second = xml
    .find("</inline-block>")
    .unwrap_or_else(|| panic!("no forest tree: {xml}"));
  assert_element(
    &xml[second..],
    "inline-block",
    &[r#"class="ltx_forest_tree""#],
    r#"<inline-block class="ltx_forest_tree">
      <inline-enumerate class="ltx_forest">
        <inline-item class="ltx_forest_node">
          <text class="ltx_forest_node_content">A</text>
          <inline-enumerate class="ltx_forest_children">
            <inline-item class="ltx_forest_node">
              <text class="ltx_forest_edge_label">dp</text>
              <text class="ltx_forest_node_content">B</text>
            </inline-item>
          </inline-enumerate>
        </inline-item>
      </inline-enumerate>
    </inline-block>"#,
  );
  assert!(!xml.contains("gone") && !xml.contains("hidden"), "{xml}");
  if let Some(n) = rng_error_count(&xml) {
    assert_eq!(n, 0, "schema-invalid: {xml}");
  }
}

/// A tree that uses the bracket parser's action character
/// (`\bracketset{action character=@}`, forest.sty:1419, 1491) is not read as
/// forest reads it: forest-doc.tex:1784-1795's `\x` builds the phantom
/// root's children through `@@` and `\bracketResume` (forest.sty:1622-1623,
/// 1450), which the token-level parser takes as the root's label. `phantom`
/// is not applied there, so ×1 … ×6 and f o r e s t — what pdflatex prints —
/// stay in the output (P6 dropped them with the phantom root's label).
/// PARTIAL, and pinned as such: pdflatex draws 13 nodes on two tiers and
/// prints no `@` and no brackets, while this output is ONE node whose label
/// `@@[×1[f]]@@[×2[o]]…` carries the action characters and the bracket
/// text as well. The six warnings are the math parser's: a lone
/// `\times_{n}` is unparsed. CONTROL: an action character set as a letter
/// `@` (prooftrees.sty:940, neoschool.cls:8568) does not match the
/// document's other `@` (`\ifx`, forest.sty:1491), so that tree is not
/// marked and its phantom root stays hidden (pdflatex prints only "kid").
#[test]
fn forest_action_character_tree_keeps_its_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/forest_action_character_phantom.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 6, "{stderr}");
  let math = |n: usize| {
    format!(
      r#"<Math class="ltx_math_unparsed" mode="inline" tex="\times_{{{n}}}" xml:id="p1.m{n}">
        <XMath>
          <XMTok meaning="times" role="MULOP">×</XMTok>
          <XMApp role="POSTSUBSCRIPT" scriptpos="1">
            <XMTok fontsize="70%" meaning="{n}" role="NUMBER">{n}</XMTok>
          </XMApp>
        </XMath>
      </Math>"#
    )
  };
  let label: String = ["f", "o", "r", "e", "s", "t"]
    .iter()
    .enumerate()
    .map(|(i, c)| format!("@@[{}[{c}]]", math(i + 1)))
    .collect();
  assert_element(
    &xml,
    "inline-item",
    &[r#"class="ltx_forest_node""#],
    &format!(
      r#"<inline-item class="ltx_forest_node">
        <text class="ltx_forest_node_content">{label}</text>
      </inline-item>"#
    ),
  );
  let control = xml
    .find("</inline-block>")
    .unwrap_or_else(|| panic!("no forest tree: {xml}"));
  assert_element(
    &xml[control..],
    "inline-block",
    &[r#"class="ltx_forest_tree""#],
    r#"<inline-block class="ltx_forest_tree">
      <inline-enumerate class="ltx_forest">
        <inline-item class="ltx_forest_node ltx_forest_phantom">
          <inline-enumerate class="ltx_forest_children">
            <inline-item class="ltx_forest_node">
              <text class="ltx_forest_node_content">kid</text>
            </inline-item>
          </inline-enumerate>
        </inline-item>
      </inline-enumerate>
    </inline-block>"#,
  );
  assert!(!xml.contains("root"), "{xml}");
}

/// A forest node as the binding emits it: its label and, when it has any,
/// its children's list.
fn forest_node(label: &str, children: &[String]) -> String {
  let children = if children.is_empty() {
    String::new()
  } else {
    format!(
      r#"<inline-enumerate class="ltx_forest_children">{}</inline-enumerate>"#,
      children.concat()
    )
  };
  format!(
    r#"<inline-item class="ltx_forest_node">
      <text class="ltx_forest_node_content">{label}</text>{children}
    </inline-item>"#
  )
}

/// `root` over the leaves `leaves`.
fn forest_star(root: &str, leaves: &[&str]) -> String {
  let leaves: Vec<String> = leaves.iter().map(|l| forest_node(l, &[])).collect();
  forest_node(root, &leaves)
}

/// Converts a keylist-runaway repro and checks what every runaway guard
/// asserts: it finishes (the pre-fix code took from 16 s to hours, or ran
/// out of memory; the bound is generous for CI's unoptimized build), with
/// exactly one error carrying `message` and no warning, and the tree is
/// still emitted as the single root node `root`.
fn assert_keylist_runaway(tex: &str, message: &str, root: &str) {
  let started = std::time::Instant::now();
  let (stderr, xml) = convert_args(tex, &[]);
  let elapsed = started.elapsed();
  assert!(elapsed.as_secs() < 60, "took {elapsed:?}: {stderr}");
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains(message), "{stderr}");
  assert_element(
    &xml,
    "inline-block",
    &[r#"class="ltx_forest_tree""#],
    &format!(
      r#"<inline-block class="ltx_forest_tree">
        <inline-enumerate class="ltx_forest">{root}</inline-enumerate>
      </inline-block>"#
    ),
  );
}

const TEN_LEAVES: [&str; 9] = ["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];

/// A style that invokes itself twice (`a/.style={a,a}`) expanded ~2^64
/// times when the nesting bound only cut each branch (P6 timed out at
/// 180 s). Its first branch now hits the 64-level nesting bound, and the
/// recorded runaway stops every later expansion in the tree: ~64
/// expansions and one error naming the depth limit. pdflatex: "TeX
/// capacity exceeded" (fatal).
#[test]
fn forest_style_runaway_stops_with_one_error() {
  assert_keylist_runaway(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/forest_style_runaway_budget.tex"
    ),
    "Error:misdefined:a forest style 'a' nests more than 64 styles deep (it invokes itself); \
     style expansion stopped",
    &forest_node("x", &[]),
  );
}

/// A style whose value doubles at each level (`a/.style={a={#1#1}}` used as
/// `a=y`) nests only one style per level, so the 64-level bound would let
/// it build 2^64 tokens; the work budget (1000000 units plus 10000 for this
/// one node) runs out at about depth 18, with one error naming the budget.
/// pdflatex: "TeX capacity exceeded" (fatal).
#[test]
fn forest_style_value_growth_trips_the_budget() {
  assert_keylist_runaway(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/forest_style_value_growth_budget.tex"
    ),
    "Error:misdefined:a forest key 'a' ran out of the keylist work budget (the tree's 1000000 \
     units plus 10000 per node, 1 node:",
    &forest_node("x", &[]),
  );
}

/// Fan-out over a large body without a value: `d16` uses the 2,000-key
/// `big` 2^17 times. Charged one unit per expansion (22d7fae7e7), `d12`
/// already took 16 s on these 10 nodes; paid per key and per token copied,
/// each use of `big` costs ~8,000 units and the pool is spent at the root
/// after about 136 uses. pdflatex stops on its own error (the bare TikZ `x` key
/// has no value).
#[test]
fn forest_style_fanout_pays_for_its_body() {
  assert_keylist_runaway(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/forest_style_fanout_budget.tex"
    ),
    "Error:misdefined:big forest key 'big' ran out of the keylist work budget (the tree's \
     1000000 units plus 10000 per node, 10 nodes:",
    &forest_star("n0", &TEN_LEAVES),
  );
}

/// A style that appends to itself (`t/.style={t/.append style={…}}`) used
/// 2^13 times per node grows by 10 keys per use: with definitions free
/// (22d7fae7e7) O(uses²) work and O(uses) memory, over 40 s for these 10
/// nodes. Every definition now pays for its new body, and the pool runs out.
#[test]
fn forest_style_append_pays_for_its_definitions() {
  assert_keylist_runaway(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/forest_style_append_budget.tex"
    ),
    "Error:misdefined:t forest key 't' ran out of the keylist work budget (the tree's 1000000 \
     units plus 10000 per node, 10 nodes:",
    &forest_star("n0", &TEN_LEAVES),
  );
}

/// A large value replayed down a deep tree: `v0`…`v15` double a value to
/// 524,288 tokens and put it in `for tree={q={…}}`, which each of the 249
/// descendants of a 250-level chain replays. 52ea71368a charged a replayed
/// key 1 unit and deep-copied the inherited keylists at every node (0
/// errors, one copy of the value per level on the recursion path; up to
/// `v19` the copies passed the 8 GB cap). The keylists now sit on one stack
/// shared by the tree, and a replayed key pays for its value at every node,
/// so the pool runs out a few levels down; every node is still emitted.
/// pdflatex: "TeX capacity exceeded" (fatal).
#[test]
fn forest_style_inherited_value_pays_at_every_node() {
  let chain = (1..250).fold(forest_node("x", &[]), |inner, _| forest_node("x", &[inner]));
  assert_keylist_runaway(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/forest_style_inherited_budget.tex"
    ),
    "Error:misdefined:q forest key 'q' ran out of the keylist work budget (the tree's 1000000 \
     units plus 10000 per node, 250 nodes:",
    &chain,
  );
}

/// Replaying the inherited `for tree` keylists stops once the budget is spent,
/// and an empty keylist is never pushed: walking the whole stack at every node
/// was unpaid work, O(nodes × stack). The root's `f7` pushes ~45,000 empty
/// keylists before its budget runs out; at 5,000 leaves the replay took 14.4 s
/// and at 15,000 about nine times that, now a few seconds. pdflatex: a
/// capacity overflow or a very long run.
#[test]
fn forest_replay_stops_when_the_budget_is_spent() {
  let repro = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/forest_style_empty_replay_budget.tex"
  );
  let more = " [x]".repeat(10_000);
  let tex = repro.replacen("[r, f7\n", &format!("[r, f7\n{more}\n"), 1);
  let leaves = ["x"; 15_000];
  assert_keylist_runaway(
    &tex,
    "Error:misdefined:f0 forest key 'f0' ran out of the keylist work budget (the tree's 1000000 \
     units plus 10000 per node, 15001 nodes:",
    &forest_star("r", &leaves),
  );
}

/// P7: every form draws its tree as an inline box (forest.sty:8506-8514 —
/// the star of `\Forest` only drops the `\forest@group@env` group, :8528;
/// forest-doc.tex:1927-1929), so `\Forest*`, `\Forest` and the `{forest}`
/// environment all emit an `ltx:inline-block` in the running text, and
/// `text \begin{forest}…\end{forest} more` stays one paragraph. A style
/// defined in a tree (`vptier`) applies in it.
#[test]
fn forest_every_form_is_an_inline_block() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/forest_starred_inline_and_libraries.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1">
      <p>We create a <inline-block class="ltx_forest_tree">
          <inline-enumerate class="ltx_forest">
            <inline-item class="ltx_forest_node">
              <text class="ltx_forest_node_content">DP</text>
              <inline-enumerate class="ltx_forest_children">
                <inline-item class="ltx_forest_node">
                  <text class="ltx_forest_node_content">D</text>
                </inline-item>
                <inline-item class="ltx_forest_node">
                  <text class="ltx_forest_node_content">NP</text>
                </inline-item>
              </inline-enumerate>
            </inline-item>
          </inline-enumerate>
        </inline-block> and merge it.</p>
    </para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2">
      <inline-block class="ltx_forest_tree">
        <inline-enumerate class="ltx_forest">
          <inline-item class="ltx_forest_node">
            <text class="ltx_forest_node_content">VP</text>
            <inline-enumerate class="ltx_forest_children">
              <inline-item class="ltx_forest_node ltx_forest_tier_vp">
                <text class="ltx_forest_node_content">V</text>
              </inline-item>
              <inline-item class="ltx_forest_node">
                <text class="ltx_forest_node_content">DP</text>
              </inline-item>
            </inline-enumerate>
          </inline-item>
        </inline-enumerate>
      </inline-block>
    </para>"#,
  );
  // The environment's preamble keys (`sn edges`, `nice empty nodes`) leave
  // no text behind.
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3">
      <p>The tree <inline-block class="ltx_forest_tree">
          <inline-enumerate class="ltx_forest">
            <inline-item class="ltx_forest_node">
              <text class="ltx_forest_node_content">S</text>
              <inline-enumerate class="ltx_forest_children">
                <inline-item class="ltx_forest_node">
                  <text class="ltx_forest_node_content">NP</text>
                </inline-item>
                <inline-item class="ltx_forest_node">
                  <text class="ltx_forest_node_content">VP</text>
                </inline-item>
              </inline-enumerate>
            </inline-item>
          </inline-enumerate>
        </inline-block> stays inline.</p>
    </para>"#,
  );
  assert!(!xml.contains("<block"), "{xml}");
  if let Some(n) = rng_error_count(&xml) {
    assert_eq!(n, 0, "schema-invalid: {xml}");
  }
}

/// P7: forest libraries named by a package option (forest.sty:140-161) or by
/// `\useforestlibrary` (:162-170) are recorded where `\ProvidesForestLibrary`
/// records them (`\forest@libraries@loaded@<lib>`, :173-177), and
/// `\forestapplylibrarydefaults` (:171-172) is defined.
#[test]
fn forest_linguistics_library_converts_clean() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/forest_starred_inline_and_libraries.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p4""#],
    r#"<para xml:id="p4"><p>LOADED LOADED</p></para>"#,
  );
}

/// Converts `tex` to HTML with the binary (`--dest t.html`, the full post
/// stage and XSLT), returning (stderr, html).
fn convert_html(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// P8: a compound's first printed use is its hyper target (chemnum.sty:1358,
/// 1373-1378), also when that use is a sub-compound spec (`\cmpd{fourth.a}`
/// writes the main label first, :1324), so every `\refcmpd` resolves to an
/// `<a href>` in the HTML.
#[test]
fn chemnum_refcmpd_resolves_in_html() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/chemnum_sub_first_main_target.tex"
  );
  let (stderr, html) = convert_html(tex);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "a",
    &[r##"href="#cmpd.first""##],
    r##"<a class="ltx_ref ltx_cmpd" href="#cmpd.first" title="">3</a>"##,
  );
  assert_element(
    &html,
    "a",
    &[r##"href="#cmpd.fourth""##],
    r##"<a class="ltx_ref ltx_cmpd" href="#cmpd.fourth" title="">5</a>"##,
  );
  assert_element(
    &html,
    "span",
    &[r#"id="cmpd.fourth""#],
    r#"<span class="ltx_text ltx_cmpd" id="cmpd.fourth"><span class="ltx_text ltx_cmpd" id="cmpd.fourth.a">5a</span></span>"#,
  );
}
