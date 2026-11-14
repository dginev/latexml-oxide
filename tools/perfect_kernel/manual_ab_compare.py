#!/usr/bin/env python3
# manual_ab_compare.py corpus.tsv outA outB: per-doc recall (core XML vs golden PDF, min-len 4)
# and Error/Fatal/Warning counts; rows that changed end in " *".
import sys,subprocess,os,re
corpus,A,B=sys.argv[1:4]
def recall(xml,pdf):
    if not (os.path.exists(xml) and os.path.exists(pdf)): return None
    r=subprocess.run(['python3',os.path.join(os.path.dirname(os.path.abspath(__file__)),'pdf_recall.py'),xml,pdf,'--min-len','4','--show','0'],capture_output=True,text=True)
    m=re.search(r'recall=([\d.]+)%',r.stdout); return float(m.group(1)) if m else None
def diag(log):
    if not os.path.exists(log): return (None,None,None)
    t=open(log,errors='replace').read()
    return (len(re.findall(r'^Error:[a-z_]+:',t,re.M)),len(re.findall(r'^Fatal:',t,re.M)),len(re.findall(r'^Warning:',t,re.M)))
rows=[]
for line in open(corpus):
    f=line.rstrip('\n').split('\t'); tex,pdf=f[1],f[2]
    b=os.path.basename(os.path.dirname(tex)); n=os.path.basename(tex)[:-4]
    ra=recall(f'{A}/{b}/{n}/{n}.xml',pdf); rb=recall(f'{B}/{b}/{n}/{n}.xml',pdf)
    da=diag(f'{A}/{b}/{n}/{n}.log'); db=diag(f'{B}/{b}/{n}/{n}.log')
    flag='' if (ra==rb and da==db) else ' *'
    print(f'{b}/{n}\trecall {ra} -> {rb}\tE/F/W {da} -> {db}{flag}')
