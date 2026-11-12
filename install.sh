#!/bin/sh
# Install the latest batisseur release binary into ~/.local/bin (no root, checksum-verified).
# Usage: curl -fsSL https://raw.githubusercontent.com/Tariq-Anjum/batisseur/main/install.sh | sh
set -eu
repo=Tariq-Anjum/batisseur
target=x86_64-unknown-linux-musl
[ "$(uname -s)" = Linux ] && [ "$(uname -m)" = x86_64 ] || { echo "batisseur: prebuilt binary is Linux x86_64 only; build from source otherwise" >&2; exit 1; }
tag=$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)
[ -n "$tag" ] || { echo "batisseur: no release found" >&2; exit 1; }
name="batisseur-$tag-$target"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
curl -fsSL -o "$tmp/$name.tar.gz" "https://github.com/$repo/releases/download/$tag/$name.tar.gz"
curl -fsSL -o "$tmp/$name.tar.gz.sha256" "https://github.com/$repo/releases/download/$tag/$name.tar.gz.sha256"
(cd "$tmp" && sha256sum -c "$name.tar.gz.sha256")
tar -xzf "$tmp/$name.tar.gz" -C "$tmp"
mkdir -p "$HOME/.local/bin"
install -m 0755 "$tmp/$name/batisseur" "$HOME/.local/bin/batisseur"
echo "batisseur $tag installed to $HOME/.local/bin/batisseur"
