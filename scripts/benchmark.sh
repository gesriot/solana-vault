#!/usr/bin/env bash
# Measure compute-unit consumption for each instruction.
# Requires solana-test-validator running locally.
set -euo pipefail

KEYPAIR=~/.config/solana/id.json
PROGRAM_ID="Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
RPC="http://localhost:8899"

echo "Starting test validator in background..."
solana-test-validator --reset --quiet &
VALIDATOR_PID=$!
sleep 3

echo "Airdropping SOL to payer..."
solana airdrop 10 --keypair $KEYPAIR --url $RPC

echo "Deploying program..."
anchor deploy --provider.cluster localnet

echo "Running tests with log capture..."
RUST_LOG=solana_runtime::system_instruction_processor=trace \
  ANCHOR_PROVIDER_URL=$RPC \
  anchor test --skip-build 2>&1 | grep -E "(Program|consumed|units)"

kill $VALIDATOR_PID
echo "Benchmark done."
