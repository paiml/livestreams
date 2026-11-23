#!/usr/bin/env bash
# rag-demo-test.sh CORPUS_DIR FILING_A FILING_B  -- each FILING is a file name in CORPUS_DIR.
# For each of the 4 show questions, `pmat query --raw --regex` must return >=1 cited line (file:line) in EACH filing.
# Prints one line per question/filing and PASS|FAIL; exit 1 on any miss. No network, never publishes.
set -u; c=${1:?corpus}; a=${2:?filing A}; b=${3:?filing B}; bad=0
Q1='off.balance|special purpose|variable interest|unconsolidated|purchase obligations|lease guarantees'
Q2='customers? (that|which|who)[^.]{0,80}(invest|financ)|investment in[^.]{0,60}customer|strategic investment|vendor financing|equity investment'
Q3='accounts receivable|receivables'
Q4='net cash provided by operating'
n=0
for q in "$Q1" "$Q2" "$Q3" "$Q4"; do n=$((n+1))
  for f in "$a" "$b"; do
    out=$(pmat query --raw --regex -i --limit 2000 "$q" -p "$c" 2>/dev/null | grep -E "^$f:[0-9]+:" | head -3)
    if [ -n "$out" ]; then echo "Q$n $f OK: $(echo "$out"|head -1|cut -c1-110)"; else echo "Q$n $f FAIL: no cited passage"; bad=1; fi
  done
done
[ $bad = 0 ] && echo "RAG-DEMO PASS" || echo "RAG-DEMO FAIL"; exit $bad
