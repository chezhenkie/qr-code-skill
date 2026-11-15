# Spec Sheet Design and Review Manual

## Purpose and principles

This manual sets out how to write, review and release a design spec sheet. The spec sheet must let design, engineering, testing, review and operations work from one source, without relying on assumptions or undocumented conversations. Every rule applies unless a gate records an approved exception.

Describe required behavior as observable outcomes, separate from implementation suggestions. Define boundaries, dependencies and ownership. Record decisions with their reasons, make ambiguous terms explicit, and prefer simple solutions. Design for failure and recovery, and make every requirement testable.

## 1. Requirements

Start with the problem: what it is, who experiences it, when it occurs, what the impact is, and why it must be solved now. Name what is out of scope. Do not describe the solution here, so the problem stays valid if the design changes. Goals are outcomes stated as results. Non-goals are related work the design deliberately excludes, which prevents scope expansion.

Write each functional requirement with a unique identifier, the required behavior, inputs, outputs, validation rules, failure behavior and any permission requirements. Use precise language. "When a valid request is submitted, the system creates one record and returns its identifier" can be tested; "The system should handle requests correctly" cannot.

Write non-functional requirements for each quality that applies, such as reliability, security, privacy, accessibility, performance, compatibility, observability, maintainability, scalability and resource usage. Name the target platforms. Hardware and operating system choices are runtime configuration, not design constraints, unless a requirement says otherwise. Every number must cite its source, a document or a measurement, or be marked as a guess. An unknown target becomes an open question, not an invented value.

Keep assumptions out of requirements, and list each one as an open question.

## 2. Architecture

Describe each component and its responsibility. State what it owns, depends on, exposes and must not do, how it communicates with others, and how it behaves when a dependency fails. Use diagrams where they clarify relationships, but keep the text complete without them.

Define the data each component owns. For each field, document its name, type, meaning, required status, allowed values, default, validation rules, sensitivity, owner, retention and migration behavior. Name the source of truth for every important value. Store a value in two places only when the design states how conflicts are resolved.

Name the component that enforces each security rule, covering authentication, authorization, input validation, secrets, encryption, logging, data exposure, abuse prevention, audit and retention. A control without an enforcement point is incomplete.

Record the alternatives for each significant choice, with benefits, costs, risks, rejected assumptions and the reason for selection or rejection. This stops old discussions from being repeated.

## 3. Interfaces

Document every boundary between components or systems, including APIs, events, files, queues, databases, command-line interfaces and user-facing controls. For each interface, define its purpose, caller, owner, request and response formats, authentication, authorization, validation, error responses, retry, timeout, idempotency and compatibility expectations.

State which changes are compatible and which are breaking, and how a breaking change is introduced. State which operations are safe to repeat and how repeats are detected. Error responses must let the caller correct the request without exposing secrets or internal detail.

## 4. Functionality

Describe the complete user journey from entry point to exit point. Cover the available actions and the default, loading, empty, validation, error, success and permission-denied states. For each state, state what the user sees and which recovery actions exist. Write this in text, because screenshots rarely show exceptional behavior.

Describe each important flow as states and transitions. For each flow, define the starting condition, trigger, processing, result, side effects, failure outcome and recovery path. Cover normal, invalid, interrupted, repeated, delayed and concurrent cases where they apply.

For each failure, state its cause, detection method, user-visible result, internal result, logged information, retry behavior, recovery action and escalation path. Errors must help the user or operator correct the problem without exposing secrets or unnecessary internal detail.

## 5. Implementation

Implement only what the approved spec defines. If the work needs behavior the spec does not cover, update the spec first, then implement. Record every intentional deviation, and keep the design and the code in agreement.

Give each component and interface a named owner, keep each component inside its stated responsibility, and keep solutions simple. Add abstraction only when a requirement needs it.

Build observability in from the start. Define the events to log, metrics to collect, traces to emit, correlation identifiers, health indicators, alerts, dashboards and retention periods. Collect no personal or sensitive data unless the spec requires it and states why.

Processes must start and stop cleanly and release every resource they acquire, so repeated runs leave no leaked memory, stale files or lingering processes.

## 6. Documentation

Every spec sheet starts with a header that records the title, purpose, owner, stakeholders, approvers, status, related documents, open questions and decision history. Status moves through draft, under review, approved, implemented and retired, as the gates in section 10 set.

Keep open questions in one register. For each one, state the decision required, the responsible person or group, the information needed and the consequence of leaving it open. Record each significant decision in the decision history with its date, reason and alternatives.

Treat the spec sheet as a living record. Update it when a requirement, decision, interface or dependency changes, when a limitation is found, when production behavior differs from the design, or when a requirement is removed. Keep it concise, but never delete information needed to understand current behavior or past decisions.

## 7. Validation

Map every requirement to at least one verification method, such as unit, integration, contract, end-to-end, accessibility, security, performance, failure and recovery, migration or manual review tests. Record the mapping in the spec so reviewers can check coverage.

Write acceptance criteria as observable statements. Each one names the starting condition, the action, the expected result, relevant edge cases and the failure behavior. Phrases such as "works as expected" or "looks good" are not acceptance criteria.

Validate the design at Gate 1, before code is written. Validate the implementation against every acceptance criterion before release, including failure, recovery, migration and rollback paths, and record the results in the spec. Performance results must state the measured configuration and method.

## 8. Stakeholders

Name every stakeholder and their role in the header. The owner is accountable for the accuracy and status of the spec. Stakeholders are the people and groups affected by the design, such as users, engineering, testing, security, privacy, operations and support. Name the approvers for each gate in the header as well.

Each component, interface and open question has a named owner who maintains it and answers questions about it. An item without an owner cannot pass a gate.

## 9. Phases

Work moves through five phases. Each phase closes at a gate, and the next phase starts only after that gate passes.

Define covers the problem, goals, non-goals, stakeholders and draft requirements. Design covers the architecture, interfaces, functionality, data, security, errors, observability and alternatives. Both phases close at Gate 1.

Build implements the design in independent increments. Each increment states its scope, dependencies, expected result, validation method and any migration or rollout impact. Progress must show as working behavior or verified artifacts, not as internal tasks alone. Each increment closes at Gate 2.

Release completes final verification, then rolls out in stages with monitoring and cleanup. The release plan states the release order, feature controls, backward compatibility, data migration, rollback conditions and rollback procedure. A rollback plan is incomplete unless it states what happens to data created by the new behavior. Gate 3 must pass before rollout starts, and Gate 4 closes after cleanup.

Operate covers monitoring, keeping the spec current, and retiring the feature when it is no longer used.

## 10. Gates

A gate passes only when its approvers confirm that every item in its list is met and recorded in the spec. A failed item returns the work to the phase that owns it. An exception needs a recorded decision with an owner and a reason.

Gate 1, design approval, closes the Define and Design phases. The status is under review while the gate runs and becomes approved when it passes. Confirm that:

- The problem, scope, goals and non-goals are clear.
- Requirements describe observable behavior, with inputs, outputs and failure cases.
- Data ownership, validation, interfaces and dependencies are documented.
- Security, privacy, user states and recovery paths are covered.
- Operational behavior is observable, and alternatives and decisions are recorded.
- Every requirement has a planned verification method, and every open question has an owner.
- Another qualified person could implement the design without private context.

Gate 2, increment acceptance, closes each Build increment. Confirm that its acceptance criteria pass as observed, its mapped tests have run and passed, its deviations are recorded or the spec is updated, and its migration or rollout impact is documented.

Gate 3, release approval, must pass before rollout starts. Confirm that all acceptance criteria pass, including failure and recovery tests; that the rollout order, feature controls, compatibility, data migration, rollback conditions and rollback procedure are defined and tested; that monitoring and alerts are in place; and that open questions affecting the release are closed.

Gate 4, closure, follows cleanup. Confirm that the spec matches production behavior and that any difference is recorded. The status then moves to implemented, and to retired when the feature is removed.
