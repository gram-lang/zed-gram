#!/bin/sh
# Copies the Zed queries of tree-sitter-gram (queries/zed/*.scm), at the
# repository and rev of extension.toml, into languages/gram/. They are
# generated there from one role table: never edit them here.
#
#   scripts/queries.sh           # copy
#   scripts/queries.sh --check   # fail if languages/gram/ differs
#
# TREE_SITTER_GRAM points to another clone, a local path included.
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
grammar="$(sed -n '/^\[grammars\.gram\]/,/^\[/p' "$root/extension.toml")"
repo="${TREE_SITTER_GRAM:-$(printf '%s\n' "$grammar" | sed -n 's/^repository = "\(.*\)"/\1/p')}"
rev="$(printf '%s\n' "$grammar" | sed -n 's/^rev = "\([0-9a-f]*\)".*/\1/p')"
[ -n "$repo" ] && [ -n "$rev" ] || { echo "no [grammars.gram] repository and rev in extension.toml" >&2; exit 1; }

src="$(mktemp -d)"
trap 'rm -rf "$src"' EXIT
git -C "$src" init -q
git -C "$src" fetch -q --depth 1 "$repo" "$rev"
git -C "$src" checkout -q FETCH_HEAD

dest="$root/languages/gram"
if [ "${1:-}" = "--check" ]; then
	status=0
	for file in "$src"/queries/zed/*.scm; do
		name="$(basename "$file")"
		cmp -s "$file" "$dest/$name" || { echo "out of date: languages/gram/$name"; status=1; }
	done
	for file in "$dest"/*.scm; do
		[ -e "$src/queries/zed/$(basename "$file")" ] || { echo "not in tree-sitter-gram: languages/gram/$(basename "$file")"; status=1; }
	done
	[ "$status" = 0 ] && echo "queries match tree-sitter-gram at ${rev%"${rev#???????}"}"
	exit "$status"
fi
rm -f "$dest"/*.scm
cp "$src"/queries/zed/*.scm "$dest/"
echo "queries copied from tree-sitter-gram at ${rev%"${rev#???????}"}"
