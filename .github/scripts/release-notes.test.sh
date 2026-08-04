#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "$0")"

pass=0
fail=0

# `contains <desc> <deve-conter|nao-deve-conter> <trecho> <assuntos>`
contains() {
  desc=$1
  mode=$2
  needle=$3
  subjects=$4

  out=$(printf '%b' "$subjects" | ./release-notes.sh)
  if printf '%s' "$out" | grep -qF "$needle"; then
    got=contem
  else
    got=nao-contem
  fi

  if [ "$got" = "$mode" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n  esperado: %s [%s]\n  saída:\n%s\n' "$desc" "$mode" "$needle" "$out"
  fi
}

# `linha_exata <desc> <linha-inteira> <assuntos>`
#
# Existe porque `contains` é frouxo demais pra cobrir a ÚNICA transformação
# que o release-notes.sh faz. Procurar por "pasta observada" casa igualmente
# em `- feat(watch): pasta observada` (linha crua, sem normalização nenhuma),
# e procurar por "watch" casa até no `feat(watch):` intacto. Ou seja: dá pra
# arrancar o sed inteiro e a suíte continua verde. Comparar a LINHA INTEIRA
# (`grep -x`) é o que trava a forma final.
linha_exata() {
  desc=$1
  linha=$2
  subjects=$3

  out=$(printf '%b' "$subjects" | ./release-notes.sh)
  # `--` obrigatório: toda linha esperada começa com o hífen do markdown, e
  # sem ele o grep leria "- watch: ..." como opção.
  if printf '%s\n' "$out" | grep -qxF -- "$linha"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n  esperava a linha exata: %s\n  saída:\n%s\n' "$desc" "$linha" "$out"
  fi
}

contains "feat vira Novidades"        contem     "### Novidades"   'feat: pasta observada\n'
contains "feat aparece sem o prefixo" contem     "pasta observada" 'feat: pasta observada\n'
contains "fix vira Correções"         contem     "### Correções"   'fix: corrige guard\n'
contains "perf vira Performance"      contem     "### Performance" 'perf: fora do mutex\n'
contains "i18n vira Traduções"        contem     "### Traduções"   'i18n: traduz\n'
contains "chore não aparece"          nao-contem "sobe dependência" 'chore: sobe dependência\n'
contains "docs não aparece"           nao-contem "ajusta README"   'docs: ajusta README\n'
contains "seção vazia não é impressa" nao-contem "### Correções"   'feat: a\n'
contains "escopo é preservado"        contem     "watch"           'feat(watch): pasta observada\n'
contains "bloco do Gatekeeper sempre entra" contem "xattr -dr com.apple.quarantine" 'feat: a\n'
contains "instalação sempre entra"    contem     "## Instalação (macOS)" 'fix: a\n'

# Forma final da linha, caractere a caractere: o tipo sai, o escopo vira
# prefixo `escopo: `, e o item ganha o hífen do markdown.
linha_exata "escopo vira prefixo, tipo some" '- watch: pasta observada' 'feat(watch): pasta observada\n'
linha_exata "sem escopo, só a descrição"     '- outra coisa'            'feat: outra coisa\n'
linha_exata "fix com escopo idem"            '- ofx: corrige guard'     'fix(ofx): corrige guard\n'
linha_exata "breaking change perde o !"      '- nova api'               'feat!: nova api\n'
linha_exata "breaking com escopo"            '- db: nova api'           'feat(db)!: nova api\n'

printf '\n%d passaram, %d falharam\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
