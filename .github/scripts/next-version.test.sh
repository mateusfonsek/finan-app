#!/usr/bin/env bash
# Testes do cálculo de bump. Shell puro de propósito: sem dependência nova,
# roda igual na máquina e no CI.
set -uo pipefail
cd "$(dirname "$0")"

pass=0
fail=0

# `input` usa \n literais, expandidos por printf %b.
check() {
  desc=$1
  current=$2
  expected=$3
  input=$4

  got=$(printf '%b' "$input" | ./next-version.sh "$current")
  if [ "$got" = "$expected" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n  esperado: [%s]\n  obtido:   [%s]\n' "$desc" "$expected" "$got"
  fi
}

check "docs sozinho não gera release"      0.2.0 ""      'docs: ajusta README\n'
check "chore sozinho não gera release"     0.2.0 ""      'chore: sobe dependência\n'
check "test/ci/refactor não geram release" 0.2.0 ""      'test: cobre x\nci: ajusta workflow\nrefactor: extrai helper\n'
check "entrada vazia não gera release"     0.2.0 ""      ''
check "feat sobe minor"                    0.2.0 "0.3.0" 'feat: pasta observada\n'
check "fix sobe patch"                     0.2.0 "0.2.1" 'fix: corrige guard\n'
check "perf sobe patch"                    0.2.0 "0.2.1" 'perf: varre fora do mutex\n'
check "i18n sobe patch"                    0.2.0 "0.2.1" 'i18n: traduz strings\n'
check "escopo é reconhecido"               0.2.0 "0.3.0" 'feat(watch): pasta observada\n'
check "feat! sobe major"                   0.2.0 "1.0.0" 'feat!: remove import legado\n'
check "feat(escopo)! sobe major"           0.2.0 "1.0.0" 'feat(ofx)!: remove parser antigo\n'
check "BREAKING CHANGE no corpo sobe major" 0.2.0 "1.0.0" 'feat: muda algo\n\nBREAKING CHANGE: o formato mudou\n'
check "maior bump vence: feat + fix"       0.2.0 "0.3.0" 'fix: a\nfeat: b\n'
check "maior bump vence: ordem inversa"    0.2.0 "0.3.0" 'feat: b\nfix: a\n'
check "maior bump vence: breaking + feat"  0.2.0 "1.0.0" 'feat: b\nfeat!: c\n'
check "commit malformado é ignorado"       0.2.0 "0.2.1" 'arrumei umas coisas\nfix: a\n'
check "só malformado não gera release"     0.2.0 ""      'arrumei umas coisas\n'
check "parte de 0.0.0"                     0.0.0 "0.1.0" 'feat: primeiro\n'
check "patch não regride minor já achado"  0.2.0 "0.3.0" 'feat: a\nfix: b\nperf: c\n'
check "versão com números altos"           1.9.9 "1.10.0" 'feat: a\n'

printf '\n%d passaram, %d falharam\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
