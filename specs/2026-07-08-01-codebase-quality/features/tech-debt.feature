@tech-debt
Feature: Code deduplication and dead code removal

  Shared utilities must be consolidated, dead code must be removed,
  and duplicate dependencies must be eliminated.

  Scenario: Shared HTML escape utility is used across all crates
    Given the codebase with duplicated html_escape implementations
    When the shared utility is extracted
    Then all crates must use the single canonical implementation
    And no local html_escape copies must remain

  Scenario: Shared slugify utility is used across all crates
    Given the codebase with duplicated slugify implementations
    When the shared utility is extracted
    Then all crates must use the single canonical implementation

  Scenario: Shared strip_html utility is used across all crates
    Given the codebase with three different strip_html implementations
    When the shared utility is extracted
    Then all crates must use the single canonical implementation
    And the canonical version must handle script/style tags and HTML entities

  Scenario: Base template context is built by a shared helper
    Given html.rs with 8 near-identical base context construction blocks
    When the helper is extracted
    Then each page-generation method must call the shared helper
    And no duplicate context construction must remain

  Scenario: Dead crates are removed from workspace
    Given unused typstify-ui and typstify-search-wasm crates
    When the dead crates are removed
    Then the workspace must still compile
    And unused workspace dependencies must be removed

  Scenario: Duplicated search types are consolidated
    Given SimpleDocument, SimpleSearchIndex, IndexManifest, FileManifest duplicated between search crates
    When the types are moved to a shared location
    Then both crates must import from the shared location
    And no duplicate type definitions must remain

  Scenario: Unused workspace dependencies are removed
    Given leptos_meta and leptos_router declared but unused
    When the dependencies are removed
    Then the workspace must still compile
    And cargo shear must report no unused dependencies
