@dx
Feature: Developer experience improvements

  The project must have a CI pipeline for linting and testing,
  and the Justfile must be complete.

  Scenario: CI pipeline runs lint and test on PRs
    Given a repository with only deployment workflows
    When a CI workflow is added
    Then the workflow must run just lint and just test on pull requests
    And the workflow must run on pushes to main/master

  Scenario: Justfile contains all documented commands
    Given AGENTS.md referencing just bdd and just test-all
    When the Justfile is updated
    Then just bdd must be a valid target
    And just test-all must be a valid target

  Scenario: README documents correct Rust version requirement
    Given README stating "Rust 1.75+" with edition 2024
    When the README is updated
    Then the minimum Rust version must match the edition requirement
