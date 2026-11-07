# Decisions

## D-012 — Bounded agent coordination and token economy

**Status:** Adopted  
**Decision:** Use only ready, independent coder assignments; scope each assignment once; keep handoffs concise; review completed author-checked work; reuse evidence; avoid duplicate validation and routine coordination chatter. Coder role effort is `low`; the reviewer remains independently configured at `gpt-6.1-sol` / medium.  
**Rationale:** Reduce avoidable coordination and repeated context use while preserving complete P0 scope, independent review, and meaningful validation.  
**Consequence:** Do not impose a hard token budget without a user-provided number. Existing sessions launched at different settings must be replaced at the assigned settings, with a short handoff and no full-history inheritance.

## D-001 — Vanilla compatibility before performance

**Status:** Adopted  
**Decision:** Prioritize 100% vanilla Minecraft Java Edition 26.3 parity, including GUI, movement, physics, client and integrated-server simulation, network protocol, world generation, saves, and baseline rendering, before performance.  
**Rationale:** Speed that changes expected behavior does not fulfill the compatibility goal.  
**Consequence:** Compatibility claims require evidence; optimization must preserve verified behavior.

## D-002 — Normal authenticated vanilla servers

**Status:** Adopted  
**Decision:** Target ordinary authenticated vanilla servers without required server-side modifications.  
**Rationale:** This defines the intended environment and avoids special operator configuration or authentication bypasses.  
**Consequence:** Deviations must be documented as limitations.

## D-003 — Free user-facing project

**Status:** Adopted  
**Decision:** Keep project functionality free for its lifetime, with no paid tiers or paywalls.  
**Rationale:** This is an explicit product commitment.  
**Consequence:** This does not promise free hosting, hardware, connectivity, or third-party services.

## D-004 — Local user asset import

**Status:** Adopted  
**Decision:** Support user-provided local asset import while avoiding unauthorized acquisition, bundling, or redistribution of copyrighted game assets.  
**Rationale:** Keep distribution of original project code distinct from third-party content.  
**Consequence:** Local extraction is not automatically legal clearance; confirm applicable rights and terms.

## D-007 — Authentication and server policy boundary

**Status:** Adopted  
**Decision:** Ordinary authenticated server compatibility presumes legitimate account entitlement and session signing; it cannot guarantee acceptance under every third-party server's rules or policies.  
**Rationale:** Client compatibility does not grant account rights or control server operators' decisions.  
**Consequence:** Do not bypass authentication, imply universal server acceptance, or label this project as a substitute for required entitlements.

## D-008 — Local asset import is limited in scope

**Status:** Adopted  
**Decision:** Local import is not an authentication bypass and does not establish blanket legal permission to extract, use, or redistribute assets.  
**Rationale:** Asset access, account authentication, and rights are separate questions.  
**Consequence:** Keep these boundaries explicit and refer users to applicable rights-holder terms.

## D-009 — Do not infer the 26.3 protocol number

**Status:** Adopted  
**Decision:** Protocol IDs must be sourced from verified local vanilla metadata or equivalent authoritative evidence; never guess or silently bake an unverified integer into the client.  
**Rationale:** The inspected local 26.3 `version.json` contains `id`, `name`, and `stable`, but no protocol integer.  
**Consequence:** Status API accepts a caller-supplied nonnegative protocol number. No 26.3 default or parity claim exists until the version mapping is verified.

## D-010 — Status ping is not authenticated gameplay

**Status:** Adopted  
**Decision:** Keep status/ping behavior distinct from login, account authentication, encryption, compression, configuration, and play.  
**Rationale:** A server-list response does not prove login compatibility, account entitlement, or gameplay parity.  
**Consequence:** Report those phases as open work; do not label the status client playable or fully compatible.

## D-011 — Bounded transport and resolver limits

**Status:** Adopted  
**Decision:** Bound packet/string sizes, connection/read/write deadlines, and system DNS work through one bounded resolver worker and queue.  
**Rationale:** Untrusted peers, slow-drip sockets, and stuck OS DNS must not permit unbounded buffers or spawned worker growth.  
**Consequence:** A blocked platform resolver can occupy the one DNS worker; later requests fail fast as busy until it returns. Do not claim OS DNS work is cancellable.

## D-005 — Evidence-based claims and unofficial status

**Status:** Adopted  
**Decision:** State compatibility and test claims in proportion to evidence and do not imply affiliation or endorsement.  
**Rationale:** Users need accurate expectations, and the project is independent.  
**Consequence:** Record scope, conditions, and known gaps.

## D-006 — MIT license for original project code

**Status:** Adopted by project owner  
**Decision:** Use MIT for original project code, excluding third-party dependencies and user-provided assets.  
**Rationale:** MIT is permissive and consistent with free use and redistribution of original code.  
**Consequence:** MIT applies only to original Sunset code. It does not apply to user-imported assets or third-party dependencies; preserve their notices and terms. This is a project choice, not legal advice.
