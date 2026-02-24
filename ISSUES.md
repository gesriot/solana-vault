# Issue Tracker (Backlog)

This file mirrors a lightweight issue tracker for the portfolio project.

## Open

### #1 [Feature] Add owner-level multi-sig support
**Priority:** High
**Description:** Integrate with Squads Protocol to enable multi-sig vault ownership.
Currently only single-keypair owners are supported. For production DeFi use,
vaults should support N-of-M multisig via Squads-compatible authority delegation.
**Acceptance Criteria:**
- Replace owner: Signer with owner: UncheckedAccount + Squads PDA validation
- Add integration test with mock Squads authority
- Document Squads setup in README

### #2 [Test] Add explicit unit tests for daily-window rollover edge cases
**Priority:** Medium
**Description:** Test daily withdrawal limit window boundary at exactly `window_start + 86400 +/- 1s`.
Current tests don't cover the precise second when the window should reset.
**Acceptance Criteria:**
- Test: withdraw at `window_start + 86399s` (1s before rollover) — should use old window
- Test: withdraw at `window_start + 86400s` (exact rollover) — should reset window
- Use Clock manipulation or time-travel helpers

### #3 [Test] Add integration tests for Token-2022 mints
**Priority:** Medium
**Description:** Verify vault works with Token-2022 extensions (transfer-fee, permanent-delegate, etc).
Current tests only use standard SPL Token Program.
**Acceptance Criteria:**
- Create Token-2022 mint with transfer-fee extension
- Test deposit/withdraw correctly handles fee deductions
- Document Token-2022 compatibility in README

### #4 [Security Test] Add adversarial CPI harness
**Priority:** High
**Description:** Create malicious mock program that attempts to exploit CPI signer forwarding.
Verify vault correctly rejects unauthorized CPIs even when attacker controls intermediate program.
**Acceptance Criteria:**
- Write mock "evil" program that tries to forward signer privileges
- Demonstrate vault's signer checks catch the attack
- Add to CI as regression test

### #5 [Test] Add delegate revocation race condition test
**Priority:** Low
**Description:** Simulate same-slot race between remove_delegate and delegate_withdraw.
Ensure delegate cannot withdraw after owner revokes access, even in same slot.
**Acceptance Criteria:**
- Use transaction builder to pack both instructions in one tx
- Verify delegate_withdraw fails when record is closed
- Document atomicity guarantees

### #6 [CI] Add compute budget regression checks
**Priority:** Medium
**Description:** Fail CI if any instruction exceeds compute unit budget thresholds.
Current benchmarks only log CU usage but don't enforce limits.
**Acceptance Criteria:**
- Parse CU consumption from test logs
- Assert: initialize < 50k, deposit < 100k, withdraw < 120k CU
- Add to GitHub Actions as separate job

### #7 [DevOps] Add release automation
**Priority:** Medium
**Description:** Automate IDL artifact upload and changelog generation on git tag push.
**Acceptance Criteria:**
- GitHub Action triggered on `v*` tags
- Upload target/idl/vault.json as release asset
- Generate changelog from commit history (conventional commits)

### #8 [Testing] Add script for deterministic benchmarking snapshots
**Priority:** Low
**Description:** Create reproducible CU measurements with fixed RNG seeds and fixed validator state.
**Acceptance Criteria:**
- Bash script that starts test-validator with deterministic slot/blockhash
- Run benchmarks 3x, verify CU values are identical
- Commit benchmark results to repo for tracking regressions

### #9 [Docs] Add operational runbook section
**Priority:** Low
**Description:** Document common operational tasks: deployment checklist, upgrade process, incident response.
**Acceptance Criteria:**
- Add OPERATIONS.md with deploy/upgrade/rollback procedures
- Include monitoring setup (RPC, compute units, error rates)
- Add incident response flowchart for common failure modes

### #10 [CI] Add devnet smoke test workflow
**Priority:** Medium
**Description:** Deploy to devnet on every PR and run basic smoke tests with real transactions.
**Acceptance Criteria:**
- GitHub Action requests devnet airdrop for ephemeral keypair
- Deploy program to devnet
- Run minimal test: initialize → deposit → withdraw → close
- Clean up devnet state after test

## Closed (v0.1.0)

1. Implement vault state PDA with canonical seed constraints.
2. Implement owner deposit/withdraw with SPL Token CPI checks.
3. Implement delegate allowance and expiry model.
4. Implement close flow with empty-vault guard.
5. Add threat model document and security considerations.
