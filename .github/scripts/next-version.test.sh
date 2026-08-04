#!/usr/bin/env bash
# Testes do cálculo de bump. Shell puro de propósito: sem dependência nova,
# roda igual na máquina e no CI.
set -uo pipefail
cd "$(dirname "$0")"

pass=0
fail=0

# `input` usa \n e \0 literais, expandidos por printf %b. \0 separa um
# commit do próximo — é o formato que o script espera (ver comentário em
# next-version.sh sobre por que a entrada é por registro, não por linha).
# `expected_exit` é conferido além do stdout: o contrato promete exit 0
# tanto no caminho releasável quanto no "sem release", e exit 1 (com stdout
# vazio) quando a versão de entrada é inválida.
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

# --- Regressão: linha de corpo não pode ser confundida com cabeçalho ---
# Este é o caso do defeito real: um commit `chore:` cujo corpo cita
# "fix: corrigido..." (ex.: colado de um changelog) não pode virar release.
# Se a separação por NUL/primeira-linha for revertida para leitura linha a
# linha, este teste falha (resultado passa a ser "0.2.1").
check "corpo com linha 'fix: algo' não é tratado como cabeçalho" \
  0.2.0 "" 0 'chore: bump deps\n\nNotas de release:\nfix: corrigido bug no changelog\n\0'

check "corpo mencionando 'chore: algo' não rebaixa feat" \
  0.2.0 "0.3.0" 0 'feat: adiciona algo\n\nchore: algo\n\0'

check "BREAKING CHANGE como última linha de corpo multi-linha sobe major" \
  0.2.0 "1.0.0" 0 'feat: muda x\n\nlinha explicando\noutra linha\nBREAKING CHANGE: quebrou tudo\n\0'

check "corpo do segundo commit citando 'feat: x' não gera release (cabeçalho é docs:)" \
  0.2.0 "" 0 'docs: a\n\0docs: b\n\nfeat: x\n\0'

# `git log --format='%B%x00'` é tformat: por não terminar em quebra de linha
# "visível" (termina no NUL), o git insere um \n extra depois de cada
# entrada. Isso faz o segundo registro em diante chegar com uma quebra de
# linha a mais na frente — simulado aqui sem precisar de um repositório real.
check "registro com \\n extra na frente (artefato do git tformat) ainda reconhece o cabeçalho" \
  0.2.0 "0.3.0" 0 'docs: a\n\0\nfeat: b\n\0'

# --- Validação da versão de entrada ---
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
