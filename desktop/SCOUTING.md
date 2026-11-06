# Scouting in the Rust Mac app

The interface stays Turkish; source identifiers, commits and developer documentation stay English. Existing role weights, the 70% coverage rule and lineup assignment are unchanged.

## Feature audit

| Requested capability | Before | Added in 0.2.0 |
| --- | --- | --- |
| CA/PA and hidden attributes | Absent from the Rust data contract and profile | Optional validated fields, a reveal switch, unknown values kept unknown |
| Advanced filters | Name, position, pagination | Combined age, ability, potential, condition, form, foot, injury, role-score and attribute ranges; sorting |
| Role fit and comparison | Role fit already implemented | Existing calculation preserved; compare up to three players in one chosen role |
| Shortlists and development | Three roster backups, without a tracking interface | Named persistent lists, unavailable members and history across game dates |

This extends the existing app-specific JSON importer. Direct `.fm`/`.fmf` parsing and live FM27 access remain unimplemented. Demo data and browser history are fictional. Actual-game compatibility still needs an import adapter and game validation.

## Backward-compatible JSON additions

`schemaVersion` stays `1`. Old files can omit all new fields.

| New root field | Contract |
| --- | --- |
| `careerId` | Optional stable identifier, 1–100 non-control characters. Supplied by the data producer; reuse only for the same career/branch, not simply the same club. |
| `gameDate` | Optional in-game date, `YYYY-MM-DD`; separate from real-world `capturedAt`. |

Optional additions to a complete player record:

```json
{
  "currentAbility": 135,
  "potentialAbility": 170,
  "hiddenAttributes": {
    "professionalism": 16,
    "consistency": 14,
    "importantMatches": 12,
    "injuryProneness": 7
  }
}
```

Ability fields are integers from 1 to 200, or omitted/null. Hidden values are 1–20 and use `HIDDEN_ATTRIBUTES` keys in `model.rs`. Omit unknown values; zero is invalid. No future attributes or transfer prices are inferred. PA is a ceiling, not a guarantee. The reveal switch controls display; it does not reproduce FM scouting knowledge/masking. AI context continues to include available player data when the user sends a message.

## Identity and persistence

- SQLite scopes lists/history by career. Demo and manual data are separate, even with matching identifiers.
- An unidentified manual import gets its own snapshot scope. Lists survive restart but do not silently merge into another import. Development requires `careerId` and `gameDate`.
- Player ID and name must match for continuity. A changed name is treated conservatively as a different identity. Missing/recycled members stay listed as unavailable and can be removed.
- Maximum 20 lists per scope, 5,000 entries per list. Add/remove is idempotent. Only empty additional lists can be deleted.
- History retains 24 distinct game dates per player independently of the three roster backups. A same-date import updates one point. Loading an earlier game date hides later points.
- Roster/history writes share one transaction; failure leaves the saved and published roster intact. Stored history is bounded and validated before display.

## Interface

**Kadro:** advanced filters, sorting, destination list, three-player comparison. **Player profile:** list membership, optional hidden data and development with an attribute selector. **Takip listeleri:** choose/create lists, browse members, compare candidates and remove unavailable entries.

Browser list mutations/imports are disabled. Its rows, role results and example history come from Rust fixtures; it does not pretend to persist edits or generate AI replies.

## Verification

Tests cover legacy imports, invalid/missing values, combined filters, unchanged role scores, Turkish search, unknown sorting, restart persistence, career isolation, recycled IDs, same-date updates, historical retention and failed transactions. Browser tests cover controls, the three-player limit, hidden data, development and preview limitations. Native Mac build/WebView checks are separate from actual-game verification.
