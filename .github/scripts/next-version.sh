#!/usr/bin/env bash
# Calcula a próxima versão a partir de mensagens de commit no padrão
# conventional commits.
#
# Lê da ENTRADA PADRÃO em vez de chamar `git` por dentro: assim os testes
# mandam texto fixo, sem precisar montar um repositório de fixture por caso.
# Quem faz o `git log` é o workflow, que liga os dois por pipe.
#
# A entrada é UM REGISTRO POR COMMIT, separado por NUL (\0) — não por linha.
# Corpo de commit é texto livre: uma linha de changelog colada, um bullet
# "fix: revisitar depois" ou qualquer rótulo em minúsculas seguido de dois-
# pontos no meio do corpo bate no regex de cabeçalho se a gente ler linha a
# linha sem separar por commit primeiro. Com NUL, cada `read` pega um commit
# inteiro; só a PRIMEIRA linha desse bloco é tratada como cabeçalho, e o
# resto é só varrido atrás do marcador BREAKING CHANGE. Isso é o que garante
# que corpo nunca vira cabeçalho — não simplifique isso de volta pra
# linha-a-linha, é exatamente o bug que essa separação corrige.
#
#   git log --format='%B%x00' v0.2.0..HEAD | next-version.sh 0.2.0
#
# Imprime a versão nova, ou NADA quando nenhum commit é releasável — é isso
# que impede um merge só de documentação de virar uma release.
set -euo pipefail

current=${1:?uso: next-version.sh <versao-atual>}

# Valida a versão de entrada antes de qualquer cálculo. Sem isso, um `v` na
# frente (a forma mais comum de passar uma tag de git por engano) ou um
# "X.Y" incompleto passam pelo `read` de baixo e produzem uma versão errada
# em silêncio — ou um erro solto no stderr que ninguém confere. Rejeitar
# aqui, alto e claro, é a única defesa.
if [[ ! $current =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  printf 'erro: versão atual inválida: "%s" (esperado X.Y.Z, sem "v" na frente)\n' "$current" >&2
  exit 1
fi

IFS=. read -r major minor patch <<< "$current"

# Precedência: major > minor > patch > none. Nunca regride.
bump=none

# `-d ''` faz o `read` parar em NUL em vez de newline: é isso que separa um
# commit do próximo. O `|| [ -n "$record" ]` cobre o caso de a entrada não
# terminar com NUL (último registro sem terminador) — mesmo truque que se
# usava com linhas, só que agora por registro. Funciona em bash 3.2 (sem
# mapfile/readarray/declare -A).
while IFS= read -r -d '' record || [ -n "$record" ]; do
  # `git log --format='%B%x00'` é tformat: como o formato não termina numa
  # quebra de linha "visível" (termina no NUL), o git insere um \n extra
  # depois de cada entrada como terminador de registro. Isso faz cada
  # registro, exceto o primeiro, chegar aqui com uma quebra de linha a mais
  # na frente. Sem remover essa quebra, a primeira linha "de verdade" cai no
  # corpo e o cabeçalho fica vazio — todo commit a partir do segundo seria
  # ignorado. Remove só essa quebra (não-op se não houver, como no primeiro
  # registro).
  record=${record#$'\n'}

  # A primeira linha do registro é o cabeçalho do commit; o resto é
  # corpo/rodapé, onde só procuramos o marcador de breaking change.
  header=${record%%$'\n'*}
  if [ "$header" = "$record" ]; then
    body=
  else
    body=${record#*$'\n'}
  fi

  # BREAKING CHANGE pode estar em qualquer linha do corpo/rodapé, sozinha.
  while IFS= read -r body_line || [ -n "$body_line" ]; do
    if [[ $body_line =~ ^BREAKING[[:space:]-]CHANGE: ]]; then
      bump=major
    fi
  done <<< "$body"

  # Cabeçalho conventional: tipo(escopo opcional)(! opcional): descrição.
  # O tipo aceita dígito porque `i18n` é um tipo real neste repo — com
  # `[a-z]+` o validador reprovaria commits que já estão no main.
  [[ $header =~ ^([a-z][a-z0-9]*)(\([^\)]*\))?(!)?:[[:space:]] ]] || continue

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
