#!/usr/bin/env bash
# Full local-dev setup script
set -euo pipefail

SOLANA_VERSION="1.18.22"
ANCHOR_VERSION="0.30.1"

echo "==> Installing Solana CLI $SOLANA_VERSION"
sh -c "$(curl -sSfL https://release.solana.com/v${SOLANA_VERSION}/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

echo "==> Configuring localnet"
solana config set --url localhost

if [ ! -f ~/.config/solana/id.json ]; then
  echo "==> Generating keypair"
  solana-keygen new --no-bip39-passphrase -o ~/.config/solana/id.json
fi

echo "==> Installing AVM + Anchor $ANCHOR_VERSION"
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install $ANCHOR_VERSION
avm use $ANCHOR_VERSION

echo "==> Installing JS dependencies"
npm install

echo "==> Build"
anchor build

echo "==> Done. Run: anchor test"
