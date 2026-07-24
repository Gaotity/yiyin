# Output directory writable only through a dedicated use case

The "frontend can never write filesystem paths" boundary was enforced only by omitting `output` from `PublicConfigDto`, while the `choose_output_directory` command mutated `config.output` (a bare `String` on `Config`) directly — a convention-based boundary with a backdoor. We decided the output directory is a domain value object (`OutputDirectory`, replacing the bare string), writable only through a dedicated `SetOutputDirectory` use case fed exclusively by the native dialog; the general config-update input structurally cannot carry it, and no DTO ever contains it. The security boundary becomes a type-level fact rather than a "remember not to add the field" convention.

## Consequences

`choose_output_directory` stops calling the general update path; any future inbound adapter (CLI, test harness) gets the same protection for free.
