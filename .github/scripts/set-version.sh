#!/usr/bin/env bash
# Escreve a versão nos TRÊS arquivos onde ela mora, mais o Cargo.lock.
#
# Existe como ponto único porque eles já divergiram na mão: o Cargo.toml
# ficou em 0.1.0 enquanto package.json e tauri.conf.json foram pra 0.2.0.
set -euo pipefail

v=${1:?uso: set-version.sh <versao>}

# Troca só o valor do campo "version" via regex de texto, sem fazer
# JSON.parse + stringify: re-serializar reformata o arquivo inteiro (o
# tauri.conf.json tem arrays escritos em uma linha só, tipo
# ["app", "dmg"], e o stringify do Node quebra cada item em uma linha —
# dezenas de linhas de diff por causa de UMA versão).
node -e '
  const fs = require("fs");
  const v = process.argv[1];
  for (const f of ["package.json", "src-tauri/tauri.conf.json"]) {
    const content = fs.readFileSync(f, "utf8");
    const updated = content.replace(/"version":\s*"[^"]*"/, `"version": "${v}"`);
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
