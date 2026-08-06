#!/usr/bin/env bash
# Tests for the bump calculation. Pure shell on purpose: no new dependency, it
# runs the same on a laptop and in CI.
set -uo pipefail
cd "$(dirname "$0")"

pass=0
fail=0

# `input` uses literal \n and \0, expanded by printf %b. \0 separates one
# commit from the next — the format the script expects (see the comment in
# next-version.sh on why input is per record, not per line).
# `expected_exit` is checked alongside stdout: the contract promises exit 0 on
# both the releasable path and the "no release" one, and exit 1 (with empty
# stdout) when the input version is invalid.
check() {
  desc=$1
  current=$2
  expected=$3
  expected_exit=$4
  input=$5

  got=$(printf '%b' "$input" | ./next-version.sh "$current")
  status=$?

  if [ "$got" = "$expected" ] && [ "$status" -eq "$expected_exit" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n  esperado: [%s] exit %d\n  obtido:   [%s] exit %d\n' \
      "$desc" "$expected" "$expected_exit" "$got" "$status"
  fi
}

check "docs sozinho não gera release"      0.2.0 ""      0 'docs: ajusta README\n\0'
check "chore sozinho não gera release"     0.2.0 ""      0 'chore: sobe dependência\n\0'
check "test/ci/refactor não geram release" 0.2.0 ""      0 'test: cobre x\n\0ci: ajusta workflow\n\0refactor: extrai helper\n\0'
check "entrada vazia não gera release"     0.2.0 ""      0 ''
check "feat sobe minor"                    0.2.0 "0.3.0" 0 'feat: pasta observada\n\0'
check "fix sobe patch"                     0.2.0 "0.2.1" 0 'fix: corrige guard\n\0'
check "perf sobe patch"                    0.2.0 "0.2.1" 0 'perf: varre fora do mutex\n\0'
check "i18n sobe patch"                    0.2.0 "0.2.1" 0 'i18n: traduz strings\n\0'
check "escopo é reconhecido"               0.2.0 "0.3.0" 0 'feat(watch): pasta observada\n\0'
check "feat! sobe major"                   0.2.0 "1.0.0" 0 'feat!: remove import legado\n\0'
check "feat(escopo)! sobe major"           0.2.0 "1.0.0" 0 'feat(ofx)!: remove parser antigo\n\0'
check "BREAKING CHANGE no corpo sobe major" 0.2.0 "1.0.0" 0 'feat: muda algo\n\nBREAKING CHANGE: o formato mudou\n\0'
# The footer only counts inside a conventional commit: in a non-conforming
# commit the marker may be quoted text (pasted changelog, release note).
check "BREAKING CHANGE em commit malformado é ignorado" 0.2.0 ""      0 'arrumei umas coisas\n\nBREAKING CHANGE: citado de um changelog\n\0'
check "malformado com marca não contamina os outros"    0.2.0 "0.2.1" 0 'arrumei coisas\n\nBREAKING CHANGE: citado\n\0fix: real\n\0'
# ...mas num commit conventional de qualquer tipo, a marca vale (spec).
check "BREAKING CHANGE em chore sobe major"             0.2.0 "1.0.0" 0 'chore: mexe em algo\n\nBREAKING CHANGE: de verdade\n\0'
check "maior bump vence: feat + fix"       0.2.0 "0.3.0" 0 'fix: a\n\0feat: b\n\0'
check "maior bump vence: ordem inversa"    0.2.0 "0.3.0" 0 'feat: b\n\0fix: a\n\0'
check "maior bump vence: breaking + feat"  0.2.0 "1.0.0" 0 'feat: b\n\0feat!: c\n\0'
check "commit malformado é ignorado"       0.2.0 "0.2.1" 0 'arrumei umas coisas\n\0fix: a\n\0'
check "só malformado não gera release"     0.2.0 ""      0 'arrumei umas coisas\n\0'
check "parte de 0.0.0"                     0.0.0 "0.1.0" 0 'feat: primeiro\n\0'
check "patch não regride minor já achado"  0.2.0 "0.3.0" 0 'feat: a\n\0fix: b\n\0perf: c\n\0'
check "versão com números altos"           1.9.9 "1.10.0" 0 'feat: a\n\0'
check "i18n(escopo) sobe patch"            0.2.0 "0.2.1" 0 'i18n(watch): x\n\0'
check "chore!: sobe major (breaking em tipo não-feat)" 0.2.0 "1.0.0" 0 'chore!: x\n\0'

# --- Regression: a body line must not be mistaken for a header ---
# This is the real defect's case: a `chore:` commit whose body quotes
# "fix: fixed..." (e.g. pasted from a changelog) must not become a release.
# If the NUL/first-line split is reverted to line-by-line reading, this test
# fails (the result becomes "0.2.1").
check "corpo com linha 'fix: algo' não é tratado como cabeçalho" \
  0.2.0 "" 0 'chore: bump deps\n\nNotas de release:\nfix: corrigido bug no changelog\n\0'

check "corpo mencionando 'chore: algo' não rebaixa feat" \
  0.2.0 "0.3.0" 0 'feat: adiciona algo\n\nchore: algo\n\0'

check "BREAKING CHANGE como última linha de corpo multi-linha sobe major" \
  0.2.0 "1.0.0" 0 'feat: muda x\n\nlinha explicando\noutra linha\nBREAKING CHANGE: quebrou tudo\n\0'

check "corpo do segundo commit citando 'feat: x' não gera release (cabeçalho é docs:)" \
  0.2.0 "" 0 'docs: a\n\0docs: b\n\nfeat: x\n\0'

# `git log --format='%B%x00'` is tformat: since it does not end in a "visible"
# newline (it ends at the NUL), git inserts an extra \n after each entry. That
# makes every record from the second on arrive with one leading newline too
# many — simulated here without needing a real repository.
check "registro com \\n extra na frente (artefato do git tformat) ainda reconhece o cabeçalho" \
  0.2.0 "0.3.0" 0 'docs: a\n\0\nfeat: b\n\0'

# --- Input version validation ---
check "versão com 'v' na frente é rejeitada (stdout vazio, exit != 0)" \
  v1.2.0 "" 1 'feat: x\n\0'
check "versão incompleta (sem patch) é rejeitada" \
  1.2 "" 1 'feat: x\n\0'
check "versão não numérica é rejeitada" \
  abc "" 1 'feat: x\n\0'
check "versão vazia é rejeitada" \
  "" "" 1 'feat: x\n\0'

printf '\n%d passaram, %d falharam\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
