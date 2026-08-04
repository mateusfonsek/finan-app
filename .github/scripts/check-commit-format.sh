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

fail=0

validate() {
  what=$1
  msg=$2
  if ! printf '%s' "$msg" | grep -Eq "$PATTERN"; then
    printf '::error::%s fora do padrão: %s\n' "$what" "$msg" >&2
    fail=1
  fi
}

validate "título da PR" "${PR_TITLE:?PR_TITLE não definido}"

while IFS= read -r subject || [ -n "$subject" ]; do
  [ -z "$subject" ] && continue
  # Merges que o próprio GitHub cria não seguem o padrão e não são autorais.
  case $subject in
    Merge\ *) continue ;;
  esac
  validate "commit" "$subject"
done

if [ "$fail" -ne 0 ]; then
  cat >&2 <<'EOF'

O formato aceito é:  tipo(escopo opcional): descrição

  feat: nova funcionalidade          → sobe a minor  (0.2.0 → 0.3.0)
  fix: correção de bug               → sobe a patch  (0.2.0 → 0.2.1)
  perf: melhoria de desempenho       → sobe a patch
  i18n: tradução                     → sobe a patch
  feat!: mudança incompatível        → sobe a major  (0.2.0 → 1.0.0)

  docs, chore, test, ci, refactor, build, style, revert → não geram release

Detalhes em CONTRIBUTING.md.
EOF
  exit 1
fi
