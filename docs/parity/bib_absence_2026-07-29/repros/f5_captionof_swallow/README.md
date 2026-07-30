# F5 — a single `\captionof{lstlisting}` swallows the document tail

Witness **2606.08339** (`preprint.tex`, 1601 lines). One line decides whether
the paper has a bibliography:

```
line 1104:  \captionof{lstlisting}{PROMISE.yml} \label{lst:bash}
```

Measured with `latexml_oxide --preload=ar5iv.sty` (the fleet's profile) on the
extracted source, so `refs.bib` resolves:

| line 1104 | document tail | bibliography |
|---|---|---|
| as shipped | rendered as **listing text** — `\newpage`, `\bibliographystyle{ACM-…}`, `\bibliography{refs}` appear line-numbered inside a code listing | **0 entries** |
| deleted | intact | **30 entries** |
| → `\caption{PROMISE.yml} \label{lst:bash}` | intact | **30 entries** |
| → `\label{lst:bash}` alone | intact | **30 entries** |
| → `\captionof{lstlisting}{X} \label{lst:bash}` | swallowed | 0 |
| → `\captionof{lstlisting}{PROMISE.yml}` (no `\label`) | swallowed | 0 |

So the trigger is `\captionof{lstlisting}{…}` itself. The caption text is
irrelevant and the `\label` is innocent. pdflatex renders the paper correctly
and raises nothing, and the source has **4 balanced `lstlisting` pairs** — the
runaway is ours, not the input's.

## How it was pinned

Deleting whole balanced regions from the complete file (slicing the body start
is useless — it cuts inside `figure`/`minipage` and manufactures its own
breakage): deleting the figure at 1081-1108 clears it, deleting the unrelated
figure at 1073-1077 does not, then line-by-line inside that figure.

The signal that matters is `\bibliographystyle` appearing as **literal text**
in the HTML. A marker word placed after the runaway is NOT a valid probe — it
survives *inside* the listing, so `grep` finds it either way. That mistake cost
a whole bisect pass here.

## Not reproduced standalone — do not re-attempt these

The paper has THREE `\captionof{lstlisting}` calls (1067, 1091, 1104) and only
the third breaks; deleting the first two changes nothing. None of the
following reproduce, so the trigger needs accumulated document state:

* a `figure` with two `minipage`s, each an `lstlisting` + `\captionof{lstlisting}`,
  then a `\caption` — with `caption` + `listings` loaded;
* the same, using the paper's real `\lstdefinestyle{jsonstyle}`/`{txtstyle}`
  definitions and its verbatim figure body (including the two `$CADNA_PATH`
  occurrences and `xrightmargin=3.em`);
* 1, 2, 3 and 4 successive `\captionof{lstlisting}` figures;
* deleting `\captionsetup[lstlisting]{labelformat=empty}` (1082), `\centering`
  (1095) or the figure `\label` (1107).

## Next step

Instrument the `lstlisting` end-detection and `\captionof`'s type handling on
the real file, rather than reducing further: the reduction space is exhausted,
but the one-line delta above is a reliable, fast red/green signal.
