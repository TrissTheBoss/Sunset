# Agent Workflow

## Roles

The project owner assigns three coder agents and one independent reviewer, each with the exact model and medium reasoning shown below. Governance is one of the three coders. Each agent follows its role file and corresponding persistent TOML template. The root/orchestrator assigns and integrates work, reads and researches, validates, reviews, approves, delegates, and maintains the project-wide action log. Root does not implement or fix code; implementation is owned by the assigned coders.

| Role | Model / reasoning | Authority and assigned boundary |
|---|---|---|
| Root/orchestrator | Project owner assigned; no coding | Assigns scopes, researches, reads, validates, reviews, approves, delegates, records own actions. Never codes or fixes implementation. |
| `requirements` | `gpt-6-luna` / medium | Requirements, parity inventory, and specification, within owner assignment. |
| `framework` | `gpt-6-luna` / medium | Application framework and core architecture, within owner assignment. |
| `governance` | `gpt-6-luna` / medium | Governance docs, source/decision/action records, and agent configuration. Cannot approve its own work. |
| `reviewer` | `gpt-6.1-sol` / medium | Independent review only; reports findings, never implements/fixes or self-approves. |

## Persistent configuration

The official Codex CLI schema templates are maintained under `config/codex/`: global agent settings in `config/codex/config.toml` and one TOML per role in `config/codex/agents/`. Codex discovers local project configuration under `.codex/`, but this workspace's protected path prevents creating `.codex/` files. These templates are source-of-truth for publication and should be mapped to `.codex/config.toml` and `.codex/agents/*.toml` by an authorized repository publishing path. Until the files are installed and Codex reports the project trusted/configuration loaded, do not claim persistent roles are active. Do not bypass the workspace restriction.

Coders implement only assigned deliverables. The reviewer provides independent findings and does not implement fixes or approve its own work. A finding is resolved only after the author corrects it and a separate review confirms the correction. Exact coder scopes are exclusive and the root must not edit their implementation files.

## Model hierarchy

Use only the exact model and reasoning level assigned to the active role. Do not select a different model, fall back to another tier, or change reasoning effort to work around availability. If unavailable, stop and report the failure to the owner. Role configuration follows the verified Codex CLI schema, but configuration and instructions are behavioral policy, not tamperproof security controls. Report unexpected overrides.

## Assignment and routing

Assignments identify an owner, scope, deliverables, constraints, and dependencies. Work outside scope requires owner direction. Do not create subagents or reassign work unless explicitly authorized. Send cross-scope findings to the owner rather than editing another role's deliverables.

## Review protocol

The reviewer reports concrete findings supported by file and line or reproducible evidence. Order findings by severity and include impact and a specific remedy. If no findings are identified, state scope and checks; do not imply broader validation. The reviewer must not fix findings or self-approve. The author addresses findings, then the reviewer performs a distinct follow-up.

## Activity and decisions

Each agent records its meaningful actions and decisions with date, role, action, and outcome, or supplies a precise entry to the owner. Do not claim another agent's work. Lasting choices belong in `docs/DECISIONS.md`; chronological work belongs in `docs/ACTION_LOG.md`. External claims should be traceable through `docs/SOURCES.md`.

## Completion report

At handoff, state deliverables, checks and results, limitations, and outstanding dependencies. Report unavailable models or absent configuration explicitly; never substitute silently.
