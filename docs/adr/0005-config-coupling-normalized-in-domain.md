# Config coupling normalized in the domain

An active background ratio guide makes landscape output meaningless, but that product rule lived only in the settings UI: one `structuredClone` poke mutated `backgroundRatioVisible` and `landscape` together, and the landscape switch was rendered `disabled` while the guide was on — evidence the combination was never meant to exist. A rule held together by one React handler is a convention with backdoors: a crafted `update_config` payload or a legacy import could persist the contradictory combination. We decided the coupling is a domain rule, silently normalized on every write path that can carry a user-influenced options payload: `Config::normalize` forces `landscape = false` whenever the ratio guide is active, applied by `UpdateConfig`, `SetOutputDirectory`, and legacy import before persisting. The frontend mirrors the rule in the `setBackgroundRatioVisible` edit intent so the UI never displays a contradictory state, but the domain is the enforcement.

## Consequences

Rejecting the contradictory combination was rejected: legacy configs may already store both flags, and failing validation on a state the UI itself produced would break those imports. Future option couplings belong in `Config::normalize`, not in UI handlers.
