<!--
Sync Impact Report:
Version change: 0.0.0 → 1.0.0 (initial constitution creation)
List of modified principles: N/A (all principles newly defined)
Added sections: All sections newly defined
Removed sections: N/A
Templates requiring updates: ✅ plan-template.md, ✅ spec-template.md, ✅ tasks-template.md (all aligned with new principles)
Follow-up TODOs: None
-->

# NightHub Constitution

## Core Principles

### I. Test-Driven Development (NON-NEGOTIABLE)
TDD is mandatory for all development: Tests MUST be written first, user MUST approve requirements, tests MUST fail, then implementation proceeds. Red-Green-Refactor cycle is strictly enforced. No code may be written without failing tests first.

### II. Readability and Simplicity
Code MUST prioritize readability over cleverness. Choose straightforward solutions over complex ones. Follow Rust idioms but avoid overly complex patterns. Code should be easily understood by experienced Rust developers without extensive mental gymnastics.

### III. Iterative Development
Work MUST be broken into small, testable chunks. Each iteration MUST advance the project incrementally with working, tested code. Large features MUST be decomposed into independently deliverable pieces.

### IV. Documentation-First
All documentation MUST be created in the docs folder with four-digit prefixes for ordering. Documentation MUST be written before or alongside code, never as an afterthought. Always use latest versions of crates and libraries.

### V. External Code Usage
Code in .external folder is for documentation purposes only and MUST NOT be modified. All dependencies MUST be installed from normal sources (crates.io), never from local .external folder. External code serves only as reference material.

## Development Standards

### Rust Quality Standards
All Rust code MUST follow best practices: proper error handling with Result types, memory safety without unsafe blocks unless absolutely necessary, and comprehensive test coverage using cargo test. Code MUST pass clippy lints with no warnings.

### Testing Requirements
Unit tests MUST cover all public functions. Integration tests MUST verify component interactions. All tests MUST be deterministic and repeatable. Test names MUST clearly describe what they verify.

## Development Workflow

### Feature Development Process
1. Write failing tests based on requirements
2. Get user approval of test scenarios
3. Implement minimal code to make tests pass
4. Refactor while maintaining test coverage
5. Create/update documentation in docs folder
6. Verify all tests pass with cargo test

### Code Review Standards
All changes MUST be reviewed for compliance with constitution principles. Reviewers MUST verify TDD was followed, documentation exists, and code maintains readability standards. Complexity MUST be justified with clear rationale.

## Governance

This constitution supersedes all other development practices. Amendments require documentation, team approval, and migration plan. All pull requests and reviews MUST verify compliance with these principles. Complexity beyond simple solutions MUST be explicitly justified and documented.

**Version**: 1.0.0 | **Ratified**: 2025-10-26 | **Last Amended**: 2025-10-26
