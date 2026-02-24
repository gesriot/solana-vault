# Testing Strategy

## Scope

This project uses three test layers:

1. Integration tests (`tests/vault.ts`) against local validator via Anchor.
2. Property-based tests (`tests/vault_property.ts`) for arithmetic and limit invariants.
3. Fuzzing (`cargo +nightly fuzz run fuzz_vault`) for malformed input exploration.

## Run Locally

```bash
npm ci
anchor test
```

Property tests only:

```bash
npx ts-mocha -p tsconfig.json tests/vault_property.ts
```

Fuzz target:

```bash
cargo +nightly fuzz run fuzz_vault
```

## Coverage Focus

- Positive flows: initialize, deposit, withdraw, delegate lifecycle, close.
- Negative flows: unauthorized signer, zero amount, over-limit, insufficient funds, expired delegate.
- Security invariants: PDA seed checks, delegate allowance monotonic usage, arithmetic overflow protection.

## Notes

- Current tests prioritize behavioral correctness and common attack paths.
- For production hardening, add mutation testing and dedicated adversarial CPI harnesses.
