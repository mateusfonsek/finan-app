#!/usr/bin/env bash
# Valida o título da PR e os assuntos de commit contra conventional commits.
#
# Existe porque a mensagem de commit virou o INPUT do versionamento: um
# formato inválido sem gate não daria erro, daria silêncio — "nenhuma release
# saiu e ninguém entendeu por quê", que é o pior jeito de falhar.
#
# O título da PR é validado junto porque, com squash merge, os commits da PR
# desaparecem e sobra um só, cuja mensagem É o título.
#
#   git log --format=%s base..head | PR_TITLE="feat: x" check-commit-format.sh
set -euo pipefail

# A mesma expressão vive no next-version.sh. A duplicação é deliberada: são
# dois scripts independentes, e um `source` compartilhado só pra uma constante
# acoplaria os dois sem ganho real.
PATTERN='^[a-z][a-z0-9]*(\([^)]+\))?!?: .+'

# Validar só a FORMA não basta: `feet: adiciona pasta observada` (typo de
# `feat`) casa no padrão, passa no gate, e depois não gera bump nenhum — o
# autor recebe PR verde e um merge que misteriosamente não lança nada. É o
# "nenhuma release saiu e ninguém entendeu por quê" do cabeçalho deste
# arquivo, só que com uma etapa a mais de disfarce. Daí a lista fechada.
#
# A lista mora numa variável só, usada tanto pela checagem quanto pelo texto
# de ajuda lá embaixo: se ficassem em dois lugares, um tipo novo entraria em
# um e não no outro. Espaço nas pontas pra permitir o teste de pertinência
# com `case` (bash 3.2: sem arrays associativos).
TYPES='feat fix perf i18n docs chore test ci refactor build style revert'

fail=0

validate() {
  what=$1
  msg=$2
  if ! printf '%s' "$msg" | grep -Eq "$PATTERN"; then
    printf '::error::%s fora do padrão: %s\n' "$what" "$msg" >&2
    fail=1
    return
  fi

  # Extrai o tipo: tudo antes do primeiro `(`, `!` ou `:`.
  type=${msg%%:*}
  type=${type%%\(*}
  type=${type%%!*}

  case " $TYPES " in
    *" $type "*) ;;
    *)
      printf '::error::%s usa um tipo desconhecido "%s": %s\n' "$what" "$type" "$msg" >&2
      fail=1
      ;;
  esac
}

# Valida que PR_TITLE foi definido e não está vazio, com erro em formato
# esperado pelo GitHub Actions, não bash bruto (:? falharia silenciosamente
# para ferramentas que não leem bash errors).
if [ -z "${PR_TITLE:-}" ]; then
  printf '::error::PR_TITLE não foi definido ou está vazio\n' >&2
  exit 1
fi

validate "título da PR" "$PR_TITLE"

while IFS= read -r subject || [ -n "$subject" ]; do
  [ -z "$subject" ] && continue
  # Pula apenas merges gerados automaticamente por git/GitHub que não seguem
  # o padrão: são artefatos do workflow, não commits autorais. Qualquer outro
  # texto que comece com "Merge" (ex: "Merge stuff") deve falhar normalmente.
  case $subject in
    "Merge pull request #"*) continue ;;
    "Merge branch "*)        continue ;;
    "Merge remote-tracking branch "*)  continue ;;
  esac
  validate "commit" "$subject"
done

if [ "$fail" -ne 0 ]; then
  # Heredoc SEM aspas no delimitador de propósito: a lista de tipos vem da
  # mesma variável que a checagem usa. Reescrever a lista aqui na mão faria
  # a mensagem e o gate divergirem no dia em que um tipo novo entrasse.
  cat >&2 <<EOF

O formato aceito é:  tipo(escopo opcional): descrição

  feat: nova funcionalidade          → sobe a minor  (0.2.0 → 0.3.0)
  fix: correção de bug               → sobe a patch  (0.2.0 → 0.2.1)
  perf: melhoria de desempenho       → sobe a patch
  i18n: tradução                     → sobe a patch
  feat!: mudança incompatível        → sobe a major  (0.2.0 → 1.0.0)

  Os demais tipos aceitos são válidos, mas não geram release.

Tipos aceitos: $TYPES

Detalhes em CONTRIBUTING.md.
EOF
  exit 1
fi
