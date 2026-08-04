#!/usr/bin/env bash
# Calcula a próxima versão a partir de mensagens de commit no padrão
# conventional commits.
#
# Lê da ENTRADA PADRÃO em vez de chamar `git` por dentro: assim os testes
# mandam texto fixo, sem precisar montar um repositório de fixture por caso.
# Quem faz o `git log` é o workflow, que liga os dois por pipe.
#
#   git log --format=%B v0.2.0..HEAD | next-version.sh 0.2.0
#
# Imprime a versão nova, ou NADA quando nenhum commit é releasável — é isso
# que impede um merge só de documentação de virar uma release.
set -euo pipefail

current=${1:?uso: next-version.sh <versao-atual>}

IFS=. read -r major minor patch <<< "$current"

# Precedência: major > minor > patch > none. Nunca regride.
bump=none

while IFS= read -r line || [ -n "$line" ]; do
  # Marca de quebra de compatibilidade no corpo/rodapé do commit.
  if [[ $line =~ ^BREAKING[[:space:]-]CHANGE: ]]; then
    bump=major
    continue
  fi

  # Cabeçalho conventional: tipo(escopo opcional)(! opcional): descrição.
  # O tipo aceita dígito porque `i18n` é um tipo real neste repo — com
  # `[a-z]+` o validador reprovaria commits que já estão no main.
  [[ $line =~ ^([a-z][a-z0-9]*)(\([^\)]*\))?(!)?:[[:space:]] ]] || continue

  type=${BASH_REMATCH[1]}
  breaking=${BASH_REMATCH[3]}

  if [ -n "$breaking" ]; then
    bump=major
  elif [ "$type" = feat ] && [ "$bump" != major ]; then
    bump=minor
  elif [ "$bump" = none ]; then
    case $type in
      fix | perf | i18n) bump=patch ;;
    esac
  fi
done

case $bump in
  major) printf '%d.0.0\n' "$((major + 1))" ;;
  minor) printf '%d.%d.0\n' "$major" "$((minor + 1))" ;;
  patch) printf '%d.%d.%d\n' "$major" "$minor" "$((patch + 1))" ;;
  none) : ;; # sem saída = sem release
esac
