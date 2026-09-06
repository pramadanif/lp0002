# Bukti: execute memindahkan dana ke penerima yang disetujui (INV-7)

Run lokal selesai 2026-09-06 17:51 UTC, sequencer standalone LEZ v0.2.4,
kedua approval dibuktikan dengan RISC0_DEV_MODE=0.

config_hash   = 0x15b622a555e276095e2101c4a5bae198fb83f90ac7045e7405085a96b664c388
proposal_seed = 0x622528817d3c4ac784def4cd400792eb1f534ccc71c9d11e26940e88ec00b058

## Saldo mentah lewat JSON-RPC getAccount
```
9KywRP3VpXwiTvZ698xN4a7nSSByJiekMxVuvSW2eH8T -> balance 0 nonce 0
GfipZYjgv7cCSTnNc6VnBgPD6icVnM6XVgFEgo9BHcRF -> balance 100 nonce 1
```

## Keluaran verify_onchain
```
  config PDA   : 9KywRP3VpXwiTvZ698xN4a7nSSByJiekMxVuvSW2eH8T
  proposal PDA : J9oqYEosZVGRSQWnvQH5SwdEA8KFrjWMYwEYnuFVfeUY
  config       : 2-of-3, owner ok, rehashes to its own address
  verifier     : matches the deployed membership program (ADR-002)
  proposal     : 2 approvals of 2 required, executed, all nullifiers distinct
  FULL M       : evidence uses the full threshold, not a lowered tier (H13/W15)
  payment      : GfipZYjgv7cCSTnNc6VnBgPD6icVnM6XVgFEgo9BHcRF holds 100, covering the 100 approved (INV-7)
  treasury     : 0 left, exactly funding minus the 100 paid
  privacy      : proposal holds a count + nullifiers, no member identity (P-F2)

  VERIFIED from public chain data alone.
```
