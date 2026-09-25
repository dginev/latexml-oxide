#!/usr/bin/env python3
# arxiv_ab_compare.py <results.tsv>: totals, status better/worse, and the papers whose
# fatal/error/bib/word(>1%)/table counts moved between binaries A and B.
import collections,sys
rows=collections.defaultdict(dict)
for l in open(sys.argv[1]):
    f=l.rstrip('\n').split('\t')
    if len(f)<10: continue
    id,tag=f[0],f[1]
    rows[id][tag]=dict(rc=int(f[2]),err=int(f[3]),fatal=int(f[4]),warn=int(f[5]),bib=int(f[6]),tab=int(f[7]),words=int(f[8]),sec=int(f[9]))
both=[i for i in rows if 'A' in rows[i] and 'B' in rows[i]]
print('papers with both:',len(both))
tot=lambda t,k: sum(rows[i][t][k] for i in both)
for k in ['err','fatal','warn','bib','tab','words']: print(f'{k}: A={tot("A",k)} B={tot("B",k)}')
def st(r): return 3 if r['fatal'] else 2 if r['err'] else 1 if r['warn'] else 0
better=[i for i in both if st(rows[i]['B'])<st(rows[i]['A'])]
worse=[i for i in both if st(rows[i]['B'])>st(rows[i]['A'])]
print('status better:',len(better),'worse:',len(worse))
for name,cond in [('fatal+',lambda a,b:b['fatal']>a['fatal']),('fatal-',lambda a,b:b['fatal']<a['fatal']),('err+',lambda a,b:b['err']>a['err']),('err-',lambda a,b:b['err']<a['err']),('bib-',lambda a,b:b['bib']<a['bib']),('bib+',lambda a,b:b['bib']>a['bib']),('words-1%',lambda a,b:b['words']<a['words']*0.99),('words+1%',lambda a,b:b['words']>a['words']*1.01),('tab-',lambda a,b:b['tab']<a['tab']),('tab+',lambda a,b:b['tab']>a['tab'])]:
    ids=[i for i in both if cond(rows[i]['A'],rows[i]['B'])]
    print(f'{name}: {len(ids)}', ' '.join(f"{i}({rows[i]['A'][name.rstrip('+-1%').replace('words','words')]}->{rows[i]['B'][name.rstrip('+-1%')]})" for i in ids[:25]))
