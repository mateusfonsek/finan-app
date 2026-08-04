#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "$0")"

pass=0
fail=0

# `check <desc> <esperado:ok|erro> <titulo-da-pr> <assuntos-de-commit>`
check() {
  desc=$1
  expected=$2
  title=$3
  subjects=$4

  if printf '%b' "$subjects" | PR_TITLE="$title" ./check-commit-format.sh >/dev/null 2>&1; then
    got=ok
  else
    got=erro
  fi

  if [ "$got" = "$expected" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n  esperado: %s\n  obtido:   %s\n' "$desc" "$expected" "$got"
  fi
}

check "título e commits válidos"        ok   'feat: x'          'feat: x\n'
check "tipo com dígito (i18n) é válido" ok   'i18n: traduz'     'i18n: traduz\n'
check "escopo é válido"                 ok   'feat(watch): x'   'feat(watch): x\n'
check "breaking é válido"               ok   'feat!: x'         'feat!: x\n'
check "escopo + breaking é válido"      ok   'fix(ofx)!: x'     'fix(ofx)!: x\n'
check "título sem tipo reprova"         erro 'arrumei coisas'   'feat: x\n'
check "título sem descrição reprova"    erro 'feat:'            'feat: x\n'
check "título com tipo maiúsculo reprova" erro 'Feat: x'        'feat: x\n'
check "commit inválido reprova"         erro 'feat: x'          'arrumei coisas\n'
check "um commit inválido no meio reprova" erro 'feat: x'       'feat: a\nlixo\nfix: b\n'
check "merge commit é ignorado"         ok   'feat: x'          'Merge branch main into feat/x\nfeat: a\n'
check "linha vazia é ignorada"          ok   'feat: x'          'feat: a\n\n'
check "sem commits, só título válido"   ok   'feat: x'          ''

printf '\n%d passaram, %d falharam\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
