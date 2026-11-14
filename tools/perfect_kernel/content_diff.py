#!/usr/bin/env python3
"""Content-preservation check between two conversions of the same document.

    content_diff.py OLD.xml NEW.xml [--show N]

Compares the multiset of words in the TEXT nodes of both files (attribute values
are ignored), so a schema fix that relocated content passes and one that dropped
authored text is caught. Prints `lost`/`gained` word counts and up to N examples of
each; exit status 1 when any word was lost. Words are Unicode letter/digit runs,
case-folded. Use it before/after every structural fix on each witness: fewer schema
errors is only a win when `lost` is 0 (or every lost word is explained, e.g. a
leaked internal control-sequence name).
"""
import re
import sys
import xml.etree.ElementTree as ET
from collections import Counter

WORD = re.compile(r"[^\W_]+", re.UNICODE)


def words(path):
    try:
        root = ET.parse(path).getroot()
    except (ET.ParseError, FileNotFoundError) as e:
        sys.exit(f"cannot read {path}: {e}")
    return Counter(w.casefold() for t in root.itertext() for w in WORD.findall(t))


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    show = 12
    if "--show" in sys.argv:
        show = int(sys.argv[sys.argv.index("--show") + 1])
        args = [a for a in args if a != str(show)]
    if len(args) != 2:
        sys.exit(__doc__)
    old, new = words(args[0]), words(args[1])
    lost, gained = old - new, new - old
    print(f"old={sum(old.values())} new={sum(new.values())} "
          f"lost={sum(lost.values())} gained={sum(gained.values())}")
    if lost:
        print("  lost:  ", " ".join(f"{w}×{n}" for w, n in lost.most_common(show)))
    if gained:
        print("  gained:", " ".join(f"{w}×{n}" for w, n in gained.most_common(show)))
    sys.exit(1 if lost else 0)


if __name__ == "__main__":
    main()
