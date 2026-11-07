# Decisions

## D-001 — Vanilla compatibility before performance

**Status:** Adopted  
**Decision:** Prioritize 100% vanilla Minecraft Java Edition 26.3 parity, including GUI simulation, integrated-server protocol/save behavior, and baseline rendering, before performance.  
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
