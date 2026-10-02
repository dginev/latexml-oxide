#!/usr/bin/env python3
"""Generate the LAE/LFE fontmap tables (latexml_package/src/package/{lae,lfe}_fontmap.rs, 59r) from arabi's
default fonts' encoding vectors in TeX Live: `ararabeyes.enc` (LAE, laecmr.fd -> aealmohanadb) and
`farsiwebencoding.enc` (LFE, lfecmr.fd -> nazli), under fonts/enc/dvips/arabi/.

Usage: arabi_fontmap_tables.py ararabeyes|farsiwebencoding > table.txt
Prints the 256 `Some(...)`/`None` rows (with the glyph names as comments) and, after a MULTI line, the
multichar entries. Each glyph name maps to its base letter (contextual forms -> the letter; Arabeyes
descriptive names, AGL afii names with .isol/.fina suffixes, uniXXXX... names); ligatures -> their letters;
a slot holding the mirror image of a Bidi_Mirrored character -> the logical character; `/` <-> `\` -> the
typed character. OXIDIZED_DESIGN_DIVERGENCES #415."""
import re, sys, unicodedata
def load(p):
    s=open(p,encoding='latin-1').read()
    s=re.sub(r'%[^\n]*','',s)
    body=s[s.index('[')+1:s.index(']')]
    return re.findall(r'/([^\s/\]]+)',body)
LET={'alef':'ALEF','beh':'BEH','teh':'TEH','theh':'THEH','jeem':'JEEM','hah':'HAH','khah':'KHAH','dal':'DAL',
 'thal':'THAL','reh':'REH','zain':'ZAIN','seen':'SEEN','sheen':'SHEEN','sad':'SAD','dad':'DAD','tah':'TAH','zah':'ZAH',
 'ain':'AIN','ghain':'GHAIN','feh':'FEH','qaf':'QAF','kaf':'KAF','arabickaf':'KAF','lam':'LAM','meem':'MEEM',
 'noon':'NOON','heh':'HEH','waw':'WAW','yeh':'YEH','alefmaksura':'ALEF MAKSURA','tehmarbuta':'TEH MARBUTA',
 'peh':'PEH','tcheh':'TCHEH','jeh':'JEH','gaf':'GAF','keheh':'KEHEH','farsiyeh':'FARSI YEH','hamza':'HAMZA',
 'alefwithhamzaabove':'ALEF WITH HAMZA ABOVE','alefwithhamzabelow':'ALEF WITH HAMZA BELOW',
 'alefwithmaddaabove':'ALEF WITH MADDA ABOVE','wawwithhamzaabove':'WAW WITH HAMZA ABOVE',
 'yehwithhamzaabove':'YEH WITH HAMZA ABOVE'}
OTHER={'fathah':'ARABIC FATHA','dammah':'ARABIC DAMMA','kasrah':'ARABIC KASRA','shaddah':'ARABIC SHADDA',
 'sukun':'ARABIC SUKUN','fathatan':'ARABIC FATHATAN','dammatan':'ARABIC DAMMATAN','kasratan':'ARABIC KASRATAN',
 'arabiccomma':'ARABIC COMMA','arabicsemicolon':'ARABIC SEMICOLON','arabicquestionmark':'ARABIC QUESTION MARK',
 'arabicpercentsign':'ARABIC PERCENT SIGN','arabicfivepointedstar':'ARABIC FIVE POINTED STAR','tatweel':'ARABIC TATWEEL',
 'ornaterightparenthesis':'ORNATE RIGHT PARENTHESIS','ornateleftparenthesis':'ORNATE LEFT PARENTHESIS',
'mhmd':'ARABIC LIGATURE MOHAMMAD ISOLATED FORM',
 'sallallahou':'ARABIC LIGATURE SALLALLAHOU ALAYHE WASALLAM','hyphenminus':'HYPHEN-MINUS',
 'exclam':'EXCLAMATION MARK','numbersign':'NUMBER SIGN','dollar':'DOLLAR SIGN','ampersand':'AMPERSAND',
 'quotesingle':'APOSTROPHE','quotedbl':'QUOTATION MARK','period':'FULL STOP','colon':'COLON',
 'guillemotright':'RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK','guillemotleft':'LEFT-POINTING DOUBLE ANGLE QUOTATION MARK',
 'parenright':'RIGHT PARENTHESIS','parenleft':'LEFT PARENTHESIS','backslash':'REVERSE SOLIDUS','slash':'SOLIDUS',
 'bracketright':'RIGHT SQUARE BRACKET','bracketleft':'LEFT SQUARE BRACKET'}
# An ASCII slot holding the mirror image of its own character was drawn mirrored for right-to-left setting:
# the character typed is the logical one (Unicode bidi mirrors it again on display).
# A glyph Unicode mirrors (Bidi_Mirrored) is stored as its mirror for right-to-left setting: the logical
# character is the mirror (`(` holds parenright, `<` holds guillemotright). The font also swaps `/` and `\\`,
# which Unicode does not mirror: such a slot decodes to the character typed.
BIDI={'(':')',')':'(','[':']',']':'[','{':'}','}':'{','<':'>','>':'<','\u00AB':'\u00BB','\u00BB':'\u00AB'}
MIRROR=dict(BIDI); MIRROR['/']=chr(92); MIRROR[chr(92)]='/'
AGL={}
for l in open('/usr/local/texlive/2025/texmf-dist/fonts/map/glyphlist/glyphlist.txt'):
    if l.startswith('#'): continue
    a,c=l.strip().split(';'); AGL[a]=c.split()[0]
DIG=['zero','one','two','three','four','five','six','seven','eight','nine']
def uni(n):
    if n in ('.notdef','.nodef'): return None
    # arabi's \\llahchar follows an alef (`\\alef\\llahchar` = Allah): the glyph is lam lam shadda heh (the LFE
    # font names it uni0644064406510647); Arabeyes calls it allahisolated, but U+FDF2 would add an alef
    if n=='allahisolated': return '\u0644\u0644\u0651\u0647'
    if n in OTHER: return unicodedata.lookup(OTHER[n])
    m=re.match(r'arabicindicdigit(\w+)$',n)
    if m: return chr(0x660+DIG.index(m.group(1)))
    m=re.match(r'(?:extendedarabicindicdigit|persiandigit)(\w+)$',n)
    if m: return chr(0x6F0+DIG.index(m.group(1)))
    m=re.match(r'lamwithalef(hamzaabove|hamzabelow|maddaabove)?(final|isolatedd?)$',n)
    if m:
        return unicodedata.lookup('ARABIC LETTER LAM')+unicodedata.lookup('ARABIC LETTER '+LET['alef'+('with'+m.group(1) if m.group(1) else '')])
    if '_' in n:
        parts=[uni(p) for p in n.split('_')]
        return None if None in parts or any(p.startswith('?') for p in parts) else ''.join(parts)
    m=re.match(r'(.+)\.(isol|init|medi|fina)$',n)
    if m: return uni(m.group(1))
    if n in AGL: return chr(int(AGL[n],16))
    m=re.match(r'(\w+?)(isolated|initial|medial|final)?$',n)
    if m and m.group(1) in LET: return unicodedata.lookup('ARABIC LETTER '+LET[m.group(1)])
    if n.startswith('uni'): return ''.join(chr(int(h,16)) for h in re.findall(r'[0-9A-F]{4}',n[3:]))
    return '?'+n
enc=sys.argv[1]
names=load(f'/usr/local/texlive/2025/texmf-dist/fonts/enc/dvips/arabi/{enc}.enc')
assert len(names)==256,len(names)
import unicodedata as ud
vals=[uni(n) for n in names]
# compatibility ligatures (allah, mohammad, ...) decode to their letters, as the text says them
vals=[(ud.normalize('NFKC',v) if v and not v.startswith('?') and ud.decomposition(v[0]).startswith('<') and len(v)==1 and 0xFB50<=ord(v)<=0xFEFF else v) for v in vals]
vals=[(chr(i) if v and 32<i<127 and MIRROR.get(chr(i))==v else (BIDI[v] if v in ('\u00AB','\u00BB') and 32<i<127 else v)) for i,v in enumerate(vals)]
multi=[(i,v) for i,v in enumerate(vals) if v and not v.startswith('?') and len(v)>1]
bad=[(i,v) for i,v in enumerate(vals) if v and v.startswith('?')]
print('UNRESOLVED',bad,file=sys.stderr)
for row in range(0,256,8):
    cells=[]
    for i in range(row,row+8):
        v=vals[i]
        cells.append('None' if v is None or v.startswith('?') or len(v)>1 else f"Some('\\u{{{ord(v):04X}}}')")
    print(f'    // 0x{row:02X}-0x{row+7:02X}: '+' '.join(names[row:row+8]))
    print('    '+', '.join(cells)+(',' if row<248 else ''))
print('MULTI')
for i,v in multi:
    print(f'    {i} => "'+''.join(f'\\u{{{ord(c):04X}}}' for c in v)+'", // '+names[i])
