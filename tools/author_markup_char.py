#!/usr/bin/env python3
"""Characterize the frontmatter creator/affiliation markup a conversion emits.

Usage: authorchar.py <core.xml>   (a --nopost latexml_oxide output)
Prints a compact, stable summary of the <creator>/<contact> tree so we can
diff the current two-branch author parser against a future unified pipeline.
"""
import re, sys

def txt(s):
    return re.sub(r'\s+', ' ', re.sub(r'<[^>]+>', '', s)).strip()

def summarize(path):
    h = open(path).read()
    out = []
    # digest area: everything from first <creator to last </creator> plus keywords
    for m in re.finditer(r'<creator\b[^>]*>(.*?)</creator>', h, re.S):
        body = m.group(1)
        role = re.search(r'role="([^"]*)"', m.group(0))
        pn = re.search(r'<personname>(.*?)</personname>', body, re.S)
        pn_empty = '<personname/>' in body or (pn and not txt(pn.group(1)))
        pn_bold = bool(pn and 'font="bold"' in pn.group(1))
        name = '<EMPTY>' if pn_empty else (txt(pn.group(1))[:32] if pn else '<none>')
        contacts = []
        for c in re.finditer(r'<contact\b[^>]*role="([^"]*)"[^>]*>(.*?)</contact>', body, re.S):
            contacts.append(f'{c.group(1)}={txt(c.group(2))[:28]!r}')
        note = 'note' if '<note' in (pn.group(1) if pn else '') else ''
        flags = ' '.join(f for f in [('BOLD' if pn_bold else ''), note] if f)
        line = f'  creator[{role.group(1) if role else "?"}] name={name!r}'
        if flags: line += f' [{flags}]'
        if contacts: line += ' :: ' + ', '.join(contacts)
        out.append(line)
    # keywords / other frontmatter of interest
    for tag in ('keywords',):
        for m in re.finditer(rf'<{tag}\b[^>]*>(.*?)</{tag}>', h, re.S):
            out.append(f'  {tag}={txt(m.group(1))[:40]!r}')
    return out

if __name__ == '__main__':
    rows = summarize(sys.argv[1])
    print(f'{len(rows)} frontmatter entries')
    for r in rows:
        print(r)
