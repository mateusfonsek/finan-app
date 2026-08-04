#!/usr/bin/env bash
set -uo pipefail

# Testes para set-version.sh
# Cria um diretório temporário com fixture, executa o script, e verifica os resultados.

script_dir="$(cd "$(dirname "$0")" && pwd)"
set_version_script="$script_dir/set-version.sh"

pass=0
fail=0

# Cria um diretório temp com a estrutura dos arquivos.
setup_fixture() {
  tmpdir=$(mktemp -d)
  mkdir -p "$tmpdir/src-tauri/src"

  # package.json
  cat > "$tmpdir/package.json" << 'EOF'
{
  "name": "finan",
  "version": "0.2.0",
  "type": "module"
}
EOF

  # src-tauri/tauri.conf.json (arrays em uma linha só, como no repo de verdade)
  cat > "$tmpdir/src-tauri/tauri.conf.json" << 'EOF'
{
  "build": {
    "beforeDevCommand": "",
    "devUrl": "http://localhost:5173",
    "frontendDist": "dist"
  },
  "app": {
    "windows": [{"fullscreen": false, "height": 600, "resizable": true, "title": "finan", "width": 800}],
    "security": {"csp": null},
    "version": "0.2.0"
  }
}
EOF

  # src-tauri/src/main.rs - arquivo mínimo para cargo metadata passar
  cat > "$tmpdir/src-tauri/src/main.rs" << 'EOF'
fn main() {}
EOF

  # src-tauri/Cargo.toml com [package] e [dependencies] sections
  cat > "$tmpdir/src-tauri/Cargo.toml" << 'EOF'
[package]
name = "finan"
version = "0.2.0"
description = "A tauri app"
authors = ["you"]
license = ""
repository = ""
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35" }

[dependencies.tauri]
version = "1.4"
features = [ "shell-open" ]
EOF

  echo "$tmpdir"
}

# Limpa o diretório temp.
cleanup_fixture() {
  rm -rf "$1"
}

# Test 1: versão válida (1.0.0) reescreve os três arquivos, uma linha cada
{
  desc="versão válida (1.0.0) reescreve arquivos"
  fixture=$(setup_fixture)

  (cd "$fixture" && "$set_version_script" "1.0.0" 2>/dev/null)
  ret=$?

  if [ $ret -eq 0 ] && \
     grep -qF '"version": "1.0.0"' "$fixture/package.json" && \
     grep -qF '"version": "1.0.0"' "$fixture/src-tauri/tauri.conf.json" && \
     grep -qF 'version = "1.0.0"' "$fixture/src-tauri/Cargo.toml" && \
     grep -qF 'tokio = { version = "1.35"' "$fixture/src-tauri/Cargo.toml"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n' "$desc"
  fi

  cleanup_fixture "$fixture"
}

# Test 2: v1.2.0 é rejeitado, nenhum arquivo modificado
{
  desc="v1.2.0 é rejeitado (nenhum arquivo modificado)"
  fixture=$(setup_fixture)

  (cd "$fixture" && $set_version_script "v1.2.0" 2>/dev/null)
  ret=$?

  if [ $ret -ne 0 ] && \
     grep -qF '"version": "0.2.0"' "$fixture/package.json" && \
     grep -qF '"version": "0.2.0"' "$fixture/src-tauri/tauri.conf.json" && \
     grep -qF 'version = "0.2.0"' "$fixture/src-tauri/Cargo.toml"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n' "$desc"
  fi

  cleanup_fixture "$fixture"
}

# Test 3: 1.2 é rejeitado (incompleto), nenhum arquivo modificado
{
  desc="1.2 é rejeitado (incompleto)"
  fixture=$(setup_fixture)

  (cd "$fixture" && $set_version_script "1.2" 2>/dev/null)
  ret=$?

  if [ $ret -ne 0 ] && \
     grep -qF '"version": "0.2.0"' "$fixture/package.json" && \
     grep -qF '"version": "0.2.0"' "$fixture/src-tauri/tauri.conf.json"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n' "$desc"
  fi

  cleanup_fixture "$fixture"
}

# Test 4: bad"version é rejeitado, JSON permanece válido
{
  desc='bad"version é rejeitado, JSON válido'
  fixture=$(setup_fixture)

  (cd "$fixture" && $set_version_script 'bad"version' 2>/dev/null)
  ret=$?

  # Confere que falhou
  if [ $ret -eq 0 ]; then
    fail=$((fail + 1))
    printf 'FALHOU: %s (deveria ter falhado mas teve sucesso)\n' "$desc"
  else
    # Confere que arquivos não foram modificados
    if grep -qF '"version": "0.2.0"' "$fixture/package.json" && \
       grep -qF '"version": "0.2.0"' "$fixture/src-tauri/tauri.conf.json"; then
      # Confere que JSON permaneça válido
      if node -e "require('$fixture/package.json')" 2>/dev/null && \
         node -e "require('$fixture/src-tauri/tauri.conf.json')" 2>/dev/null; then
        pass=$((pass + 1))
      else
        fail=$((fail + 1))
        printf 'FALHOU: %s (JSON inválido)\n' "$desc"
      fi
    else
      fail=$((fail + 1))
      printf 'FALHOU: %s (arquivo foi modificado)\n' "$desc"
    fi
  fi

  cleanup_fixture "$fixture"
}

# Test 5: nenhum argumento é rejeitado
{
  desc="nenhum argumento é rejeitado"
  fixture=$(setup_fixture)

  (cd "$fixture" && $set_version_script 2>/dev/null)
  ret=$?

  if [ $ret -ne 0 ] && \
     grep -qF '"version": "0.2.0"' "$fixture/package.json"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n' "$desc"
  fi

  cleanup_fixture "$fixture"
}

# Test 6: versão válida (1.5.3) reescreve
{
  desc="versão válida (1.5.3) reescreve arquivos"
  fixture=$(setup_fixture)

  (cd "$fixture" && $set_version_script "1.5.3" 2>/dev/null)
  ret=$?

  if [ $ret -eq 0 ] && \
     grep -qF '"version": "1.5.3"' "$fixture/package.json" && \
     grep -qF '"version": "1.5.3"' "$fixture/src-tauri/tauri.conf.json" && \
     grep -qF 'version = "1.5.3"' "$fixture/src-tauri/Cargo.toml"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n' "$desc"
  fi

  cleanup_fixture "$fixture"
}

# Test 7: [package] version muda, [dependencies] versões não mudam
{
  desc="[package] version muda, [dependencies] versões não mudam"
  fixture=$(setup_fixture)

  (cd "$fixture" && $set_version_script "2.0.0" 2>/dev/null)
  ret=$?

  if [ $ret -eq 0 ] && \
     grep -qF 'version = "2.0.0"' "$fixture/src-tauri/Cargo.toml" && \
     grep -qF 'tokio = { version = "1.35"' "$fixture/src-tauri/Cargo.toml" && \
     grep -qF 'serde = { version = "1.0"' "$fixture/src-tauri/Cargo.toml"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s\n' "$desc"
  fi

  cleanup_fixture "$fixture"
}

# Test 8: Cargo.toml cuja linha de version não casa com o perl => erro
# (o `^version = ` não bate numa linha indentada). O modo de falha que este
# teste tranca é o SILENCIOSO: sem a guarda no script, o perl não substitui
# nada, sai 0, e o script termina com sucesso tendo bumpado só os JSON.
{
  desc="Cargo.toml não substituído reprova (não fica em silêncio)"
  fixture=$(setup_fixture)

  cat > "$fixture/src-tauri/Cargo.toml" << 'EOF'
[package]
  name = "finan"
  version = "0.2.0"
  edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
EOF

  (cd "$fixture" && $set_version_script "3.0.0" 2>/dev/null)
  ret=$?

  if [ $ret -ne 0 ] && grep -qF 'version = "0.2.0"' "$fixture/src-tauri/Cargo.toml"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'FALHOU: %s (saiu %d)\n' "$desc" "$ret"
  fi

  cleanup_fixture "$fixture"
}

printf '\n%d passaram, %d falharam\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
