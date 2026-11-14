use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: textgreek.sty.ltxml — not present in Perl LaTeXML.
  // The TL textgreek.sty package (Leonard Michlmayr, 2010-2011) provides
  // text-mode Greek letters via `\DeclareTextGreekSymbol` over the LGR font
  // encoding. Our raw load doesn't pick them up (LGR encoding isn't
  // installed in our font stack), so the binding maps each `\text<greek>`
  // CS to its Unicode equivalent for clean XML output.
  //
  // Witnesses (stage-1..2 of 100k warning corpus):
  //   arXiv:2603.02703 — `\textsigma` undefined via cuted.sty cascade
  //   arXiv:2604.09141 — same pattern

  // Lowercase Greek
  DefPrimitive!("\\textalpha",   "\u{03B1}");
  DefPrimitive!("\\textbeta",    "\u{03B2}");
  DefPrimitive!("\\textgamma",   "\u{03B3}");
  DefPrimitive!("\\textdelta",   "\u{03B4}");
  DefPrimitive!("\\textepsilon", "\u{03B5}");
  DefPrimitive!("\\textzeta",    "\u{03B6}");
  DefPrimitive!("\\texteta",     "\u{03B7}");
  DefPrimitive!("\\texttheta",   "\u{03B8}");
  DefPrimitive!("\\textiota",    "\u{03B9}");
  DefPrimitive!("\\textkappa",   "\u{03BA}");
  DefPrimitive!("\\textlambda",  "\u{03BB}");
  DefPrimitive!("\\textmugreek", "\u{03BC}");
  DefPrimitive!("\\textnu",      "\u{03BD}");
  DefPrimitive!("\\textxi",      "\u{03BE}");
  DefPrimitive!("\\textomikron", "\u{03BF}");
  DefPrimitive!("\\textpi",      "\u{03C0}");
  DefPrimitive!("\\textrho",     "\u{03C1}");
  DefPrimitive!("\\textsigma",   "\u{03C3}");
  DefPrimitive!("\\textvarsigma","\u{03C2}");
  DefPrimitive!("\\texttau",     "\u{03C4}");
  DefPrimitive!("\\textupsilon", "\u{03C5}");
  DefPrimitive!("\\textphi",     "\u{03C6}");
  DefPrimitive!("\\textchi",     "\u{03C7}");
  DefPrimitive!("\\textpsi",     "\u{03C8}");
  DefPrimitive!("\\textomega",   "\u{03C9}");

  // Uppercase Greek
  DefPrimitive!("\\textAlpha",   "\u{0391}");
  DefPrimitive!("\\textBeta",    "\u{0392}");
  DefPrimitive!("\\textGamma",   "\u{0393}");
  DefPrimitive!("\\textDelta",   "\u{0394}");
  DefPrimitive!("\\textEpsilon", "\u{0395}");
  DefPrimitive!("\\textZeta",    "\u{0396}");
  DefPrimitive!("\\textEta",     "\u{0397}");
  DefPrimitive!("\\textTheta",   "\u{0398}");
  DefPrimitive!("\\textIota",    "\u{0399}");
  DefPrimitive!("\\textKappa",   "\u{039A}");
  DefPrimitive!("\\textLambda",  "\u{039B}");
  DefPrimitive!("\\textMu",      "\u{039C}");
  DefPrimitive!("\\textNu",      "\u{039D}");
  DefPrimitive!("\\textXi",      "\u{039E}");
  DefPrimitive!("\\textOmikron", "\u{039F}");
  DefPrimitive!("\\textPi",      "\u{03A0}");
  DefPrimitive!("\\textRho",     "\u{03A1}");
  DefPrimitive!("\\textSigma",   "\u{03A3}");
  DefPrimitive!("\\textTau",     "\u{03A4}");
  DefPrimitive!("\\textUpsilon", "\u{03A5}");
  DefPrimitive!("\\textPhi",     "\u{03A6}");
  DefPrimitive!("\\textChi",     "\u{03A7}");
  DefPrimitive!("\\textPsi",     "\u{03A8}");
  DefPrimitive!("\\textOmega",   "\u{03A9}");

  // Archaic / extra Greek letters. greek-fontenc (loaded by textalpha) declares
  // these over LGR/TU (greek-fontenc/tuenc-greek.def L232-241,
  // lgrenc.def `\DeclareTextSymbol{\textQoppa}{LGR}{21}`); since we map the
  // `\text<greek>` family to Unicode rather than rely on the LGR font, mirror
  // the full archaic set so a paper using e.g. `\textQoppa` (witness
  // arXiv:1505.03770, `\def\Qop{\text{\rm\textQoppa}}`) doesn't hit an
  // undefined CS where Perl — which raw-loads textalpha — is clean.
  DefPrimitive!("\\textQoppa",   "\u{03D8}"); // Ϙ archaic Koppa
  DefPrimitive!("\\textqoppa",   "\u{03D9}"); // ϙ
  DefPrimitive!("\\textStigma",  "\u{03DA}"); // Ϛ
  DefPrimitive!("\\textstigma",  "\u{03DB}"); // ϛ
  DefPrimitive!("\\textDigamma", "\u{03DC}"); // Ϝ
  DefPrimitive!("\\textdigamma", "\u{03DD}"); // ϝ
  DefPrimitive!("\\textKoppa",   "\u{03DE}"); // Ϟ
  DefPrimitive!("\\textkoppa",   "\u{03DF}"); // ϟ
  DefPrimitive!("\\textSampi",   "\u{03E0}"); // Ϡ
  DefPrimitive!("\\textsampi",   "\u{03E1}"); // ϡ

  // textgreek.sty L235-260 also exposes `\straight<letter>` variants
  // (the "upright" forms used in physics typography where the default
  // italic theta is unwanted). Map to Unicode glyphs that match the
  // textgreek package's PU font encoding. Witness: arXiv:2604.15081
  // uses `\straighttheta/2\straighttheta` in hafnia-thin-film prose.
  DefPrimitive!("\\straighttheta",   "\u{03B8}"); // θ
  DefPrimitive!("\\straightphi",     "\u{03D5}"); // ϕ (PHI SYMBOL)
  DefPrimitive!("\\straightepsilon", "\u{03F5}"); // ϵ (LUNATE EPSILON SYMBOL)

  // alphabeta.sty L205ff: bare-name aliases (`\providecommand*{\Alpha}
  // {\textAlpha}` and siblings) — Greek LETTERS, content-bearing. Sweep-12
  // cluster witnesses greek-fontenc char-list-alphabeta (91 errs),
  // teubner-doc, biblatex-sbl/sbl-paper.
  DefPrimitive!("\\Alpha",      "\u{0391}");
  DefPrimitive!("\\Beta",       "\u{0392}");
  DefPrimitive!("\\Epsilon",    "\u{0395}");
  DefPrimitive!("\\Zeta",       "\u{0396}");
  DefPrimitive!("\\Eta",        "\u{0397}");
  DefPrimitive!("\\Iota",       "\u{0399}");
  DefPrimitive!("\\Kappa",      "\u{039A}");
  DefPrimitive!("\\Mu",         "\u{039C}");
  DefPrimitive!("\\Nu",         "\u{039D}");
  DefPrimitive!("\\Omicron",    "\u{039F}");
  DefPrimitive!("\\Rho",        "\u{03A1}");
  DefPrimitive!("\\Tau",        "\u{03A4}");
  DefPrimitive!("\\Chi",        "\u{03A7}");
  DefPrimitive!("\\omicron",    "\u{03BF}");
  DefPrimitive!("\\finalsigma", "\u{03C2}");
  DefPrimitive!("\\varbeta",    "\u{03D0}");
  // \providecommand semantics (alphabeta.sty L205ff): \varkappa/\digamma
  // are also amssymb MATH symbols — never clobber an existing definition.
  if !IsDefined!(&T_CS!("\\varkappa")) {
    DefPrimitive!("\\varkappa", "\u{03F0}");
  }
  if !IsDefined!(&T_CS!("\\digamma")) {
    DefPrimitive!("\\digamma", "\u{03DD}");
  }
  DefPrimitive!("\\Digamma",    "\u{03DC}");
  DefPrimitive!("\\koppa",      "\u{03DF}");
  DefPrimitive!("\\Koppa",      "\u{03DE}");
  DefPrimitive!("\\Sampi",      "\u{03E0}");
  DefPrimitive!("\\sampi",      "\u{03E1}");
  DefPrimitive!("\\Stigma",     "\u{03DA}");
  DefPrimitive!("\\stigma",     "\u{03DB}");
  DefPrimitive!("\\Qoppa",      "\u{03D8}");
  DefPrimitive!("\\qoppa",      "\u{03D9}");
  DefPrimitive!("\\thetasymbol",   "\u{03D1}");
  DefPrimitive!("\\phisymbol",     "\u{03D5}");
  DefPrimitive!("\\pisymbol",      "\u{03D6}");
  DefPrimitive!("\\rhosymbol",     "\u{03F1}");
  DefPrimitive!("\\kappasymbol",   "\u{03F0}");
  DefPrimitive!("\\betasymbol",    "\u{03D0}");
  DefPrimitive!("\\epsilonsymbol", "\u{03F5}");

  // alphabeta.sty L109-160: `\math<letter>` aliases saving the ORIGINAL math
  // meanings before alphabeta re-declares the bare names as
  // `\TextOrMath{\text<x>}{\math<x>}`. Packages built on alphabeta call these
  // directly: hep-math-font.sty L146-186 (`\hep@greek{\textdelta}{\mathdelta}`).
  // Since `alphabeta` maps to THIS binding (lib.rs), the raw let-block never
  // runs — port it verbatim. Witness: hep-paper manual (perfect-kernel,
  // \mathdelta/\mathmu/\mathphi undefined).
  RawTeX!(
    r"\let\mathGamma\Gamma \let\mathDelta\Delta \let\mathTheta\Theta
\let\mathLambda\Lambda \let\mathXi\Xi \let\mathPi\Pi
\let\mathSigma\Sigma \let\mathUpsilon\Upsilon \let\mathPhi\Phi
\let\mathPsi\Psi \let\mathOmega\Omega
\let\mathalpha\alpha \let\mathbeta\beta \let\mathgamma\gamma
\let\mathdelta\delta \let\mathepsilon\epsilon \let\mathvarepsilon\varepsilon
\let\mathzeta\zeta \let\matheta\eta \let\maththeta\theta
\let\mathvartheta\vartheta \let\mathiota\iota \let\mathkappa\kappa
\let\mathlambda\lambda \let\mathmu\mu \let\mathnu\nu
\let\mathxi\xi \let\mathpi\pi \let\mathvarpi\varpi
\let\mathrho\rho \let\mathvarrho\varrho \let\mathsigma\sigma
\let\mathvarsigma\varsigma \let\mathtau\tau \let\mathupsilon\upsilon
\let\mathphi\phi \let\mathvarphi\varphi \let\mathchi\chi
\let\mathpsi\psi \let\mathomega\omega"
  );

  // greek-fontenc accent commands (lgrenc.def L439ff \DeclareTextAccent —
  // our \DeclareTextAccent is a no-op, as in Perl). Bounded surpass: map
  // each to its Unicode COMBINING mark placed after the argument's first
  // char via \lx@applyaccent's convention is overkill here — emit
  // argument + combining mark, which NFC-normalizes downstream.
  DefMacro!("\\acctonos{}", "#1\u{0301}");
  DefMacro!("\\accvaria{}", "#1\u{0300}");
  DefMacro!("\\accperispomeni{}", "#1\u{0342}");
  DefMacro!("\\accdialytika{}", "#1\u{0308}");
  DefMacro!("\\accpsili{}", "#1\u{0313}");
  DefMacro!("\\accdasia{}", "#1\u{0314}");
  DefMacro!("\\accpsilioxia{}", "#1\u{0313}\u{0301}");
  DefMacro!("\\accdasiaoxia{}", "#1\u{0314}\u{0301}");
  DefMacro!("\\accpsilivaria{}", "#1\u{0313}\u{0300}");
  DefMacro!("\\accdasiavaria{}", "#1\u{0314}\u{0300}");
  DefMacro!("\\accpsiliperispomeni{}", "#1\u{0313}\u{0342}");
  DefMacro!("\\accdasiaperispomeni{}", "#1\u{0314}\u{0342}");
  DefMacro!("\\accdialytikatonos{}", "#1\u{0308}\u{0301}");
  DefMacro!("\\accdialytikavaria{}", "#1\u{0308}\u{0300}");
  DefMacro!("\\accdialytikaperispomeni{}", "#1\u{0308}\u{0342}");
  DefMacro!("\\ypogegrammeni", "\u{0345}");
});
