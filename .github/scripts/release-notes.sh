#!/usr/bin/env bash
# Monta as notas da release a partir dos assuntos de commit (stdin).
#
# Só entram os tipos que geram bump. Quem abre uma release quer saber o que
# mudou pra ELE — não que uma dependência subiu ou que um teste foi coberto.
set -euo pipefail

# A entrada é lida UMA vez pra dentro da variável. Rodar quatro `grep`
# direto na stdin não funciona: o primeiro consome tudo e os outros três
# recebem entrada vazia — o bug sai silencioso, com só a primeira seção
# preenchida.
all=$(cat)

feats=$(printf '%s\n' "$all" | grep -E '^feat(\([^)]+\))?!?: ' || true)
fixes=$(printf '%s\n' "$all" | grep -E '^fix(\([^)]+\))?!?: ' || true)
perfs=$(printf '%s\n' "$all" | grep -E '^perf(\([^)]+\))?!?: ' || true)
i18ns=$(printf '%s\n' "$all" | grep -E '^i18n(\([^)]+\))?!?: ' || true)

section() {
  title=$1
  body=$2
  [ -z "$body" ] && return 0
  printf '### %s\n\n' "$title"
  # O escopo (o que vem entre parênteses, tipo `feat(watch):`) é preservado
  # na frente da linha — só o tipo do commit (feat/fix/perf/i18n) é ruído
  # pra quem lê a release. Por isso a lista de tipos é explícita: um
  # `^[a-z][a-z0-9]*` genérico bateria de novo no próprio escopo já
  # reescrito (ex.: "watch: pasta observada") e apagaria o escopo.
  printf '%s\n' "$body" | sed -E 's/^(feat|fix|perf|i18n)\(([^)]+)\)!?: /\2: /; s/^(feat|fix|perf|i18n)!?: //; s/^/- /'
  printf '\n'
}

section "Novidades" "$feats"
section "Correções" "$fixes"
section "Performance" "$perfs"
section "Traduções" "$i18ns"

cat <<'EOF'
---

## Instalação (macOS)

1. Baixe o `.dmg` abaixo, abra e arraste o **finan app** para a pasta **Aplicativos**.
2. O app **não é assinado** (projeto open-source individual), então o macOS bloqueia na primeira abertura. Remova a quarentena do Gatekeeper no Terminal:

   ```
   xattr -dr com.apple.quarantine "/Applications/finan app.app"
   ```

3. Abra normalmente. Tudo **100% local** — seus dados ficam só no seu Mac, em `~/Library/Application Support/app.finan/finan.db`.
EOF
