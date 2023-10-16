# Lithica Rationale

This is a living document intended to rationalize Lithica's existence, as well as all of its underlying design and implementation decisions.

## `any_t` and `type_t` Keywords

These keywords are idiosyncratic in that they use the `_t` suffix which is ubiquitous to C standard types, but not shared by any other part of the Lithica standard. This is deliberate; they are intended to appear visually distinct from "normal" type specifiers, since they have such wildly different semantics. `any` and `type` were passed over as candidates to preserve these signs for user code, which benefits from being able to use them in value identifiers.
