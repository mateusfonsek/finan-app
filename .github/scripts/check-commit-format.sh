#!/usr/bin/env bash
# Validates the PR title and the commit subjects against conventional commits.
#
# Exists because the commit message became the INPUT to versioning: an invalid
# format with no gate would not raise an error, it would raise silence — "no
# release came out and nobody understood why", which is the worst way to fail.
#
# The PR title is validated alongside because, with squash merge, the PR's
# commits disappear and one remains, whose message IS the title.
#
#   git log --format=%s base..head | PR_TITLE="feat: x" check-commit-format.sh
set -euo pipefail

# The same expression lives in next-version.sh. The duplication is deliberate:
# they are two independent scripts, and a shared `source` for a single constant
# would couple them with no real gain.
PATTERN='^[a-z][a-z0-9]*(\([^)]+\))?!?: .+'

# Validating only the SHAPE is not enough: `feet: add watched folder` (a typo of
# `feat`) matches the pattern, passes the gate, and then generates no bump at
# all — the author gets a green PR and a merge that mysteriously ships nothing.
# It is the "no release came out and nobody understood why" from this file's
# header, with one extra layer of disguise. Hence the closed list.
#
# The list lives in a single variable, used both by the check and by the help
# text below: were they in two places, a new type would land in one and not the
# other. Spaces at the edges allow the membership test with `case` (bash 3.2: no
# associative arrays).
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

  # Extracts the type: everything before the first `(`, `!` or `:`.
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

# Validates that PR_TITLE was set and is not empty, erroring in the format
# GitHub Actions expects rather than raw bash (:? would fail silently for tools
# that do not read bash errors).
if [ -z "${PR_TITLE:-}" ]; then
  printf '::error::PR_TITLE não foi definido ou está vazio\n' >&2
  exit 1
fi

validate "título da PR" "$PR_TITLE"

while IFS= read -r subject || [ -n "$subject" ]; do
  [ -z "$subject" ] && continue
  # Skips only the merges git/GitHub generate automatically that do not follow
  # the pattern: they are workflow artifacts, not authored commits. Any other
  # text starting with "Merge" (e.g. "Merge stuff") must fail normally.
  case $subject in
    "Merge pull request #"*) continue ;;
    "Merge branch "*)        continue ;;
    "Merge remote-tracking branch "*)  continue ;;
  esac
  validate "commit" "$subject"
done

if [ "$fail" -ne 0 ]; then
  # Heredoc with an UNQUOTED delimiter on purpose: the type list comes from the
  # same variable the check uses. Rewriting the list here by hand would make the
  # message and the gate diverge the day a new type was added.
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
