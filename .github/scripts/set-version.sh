#!/usr/bin/env bash
# Escreve a versão nos TRÊS arquivos onde ela mora, mais o Cargo.lock.
#
# Existe como ponto único porque eles já divergiram na mão: o Cargo.toml
# ficou em 0.1.0 enquanto package.json e tauri.conf.json foram pra 0.2.0.
set -euo pipefail

v=${1:?uso: set-version.sh <versao>}

# Valida a versão de entrada antes de qualquer substituição. Sem isso,
# um `v` na frente (forma mais comum de passar uma tag de git por engano)
# ou um "X.Y" incompleto passam pelo node e produzem JSON inválido em
# silêncio. Rejeitar aqui, alto e claro, é a única defesa.
if [[ ! $v =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  printf 'erro: versão inválida: "%s" (esperado X.Y.Z, sem "v" na frente)\n' "$v" >&2
  exit 1
fi

# Troca só o valor do campo "version" via regex de texto, sem fazer
# JSON.parse + stringify: re-serializar reformata o arquivo inteiro (o
# tauri.conf.json tem arrays escritos em uma linha só, tipo
# ["app", "dmg"], e o stringify do Node quebra cada item em uma linha —
# dezenas de linhas de diff por causa de UMA versão).
#
# Guarda cada substituição para garantir que a chave foi encontrada: se
# o `replace` não mudar nada, o arquivo sai intocado e a versão errada
# fica silenciosamente no lugar. O mesmo risco que o perl previne com o
# flag `$done` abaixo — deixar descoberto aqui seria repetir o mesmo bug.
node -e '
  const fs = require("fs");
  const v = process.argv[1];
  for (const f of ["package.json", "src-tauri/tauri.conf.json"]) {
    const content = fs.readFileSync(f, "utf8");
    const updated = content.replace(/"version":\s*"[^"]*"/, `"version": "${v}"`);
    if (updated === content) {
      console.error(`erro: chave "version" não encontrada em ${f}`);
      process.exit(1);
    }
    fs.writeFileSync(f, updated);
  }
' "$v"

# Só a PRIMEIRA ocorrência, que é a da seção [package] (a primeira do arquivo).
# Nunca a `version` de uma dependência.
#
# O flag só é marcado quando a substituição REALMENTE acontece — um
# `unless $done++` marcaria já na linha 1 (`[package]`) e o arquivo sairia
# intocado, em silêncio.
perl -pi -e 'if (!$done && s/^version = "[^"]*"/version = "'"$v"'"/) { $done = 1 }' src-tauri/Cargo.toml

# Ler o manifesto já reescreve o lock com a versão nova.
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 >/dev/null
