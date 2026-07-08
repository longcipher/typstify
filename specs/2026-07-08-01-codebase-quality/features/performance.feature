@performance
Feature: Build performance optimization

  The build pipeline must avoid redundant allocations, cache constant
  computations, and use references instead of clones where possible.

  Scenario: Config is not cloned unnecessarily in build pipeline
    Given a build pipeline that clones Config 8+ times
    When generators accept &Config references
    Then no unnecessary Config clones must occur
    And the build must produce identical output

  Scenario: base_url is computed once and reused
    Given a build that calls base_url() per page (allocating each time)
    When base_url is pre-computed once
    Then the build must produce identical output
    And no per-page base_url allocations must occur

  Scenario: Year string is computed once per build
    Given a build that calls Utc::now().year().to_string() per page
    When the year is computed once at build start
    Then the build must produce identical output

  Scenario: Section nav and language switcher are cached
    Given a build that regenerates section_nav and lang_switcher per page
    When the HTML strings are cached per language
    Then the build must produce identical output
    And regeneration must not occur for repeated languages

  Scenario: slug_from_str uses single-pass algorithm
    Given a slugify function with redundant collect-split-filter-join
    When replaced with a single-pass builder
    Then the output must be identical for all inputs
    And intermediate allocations must be eliminated

  Scenario: escape_xml uses single-pass algorithm
    Given an escape_xml function with 5 chained .replace() calls
    When replaced with a single-pass builder
    Then the output must be identical for all inputs
    And intermediate allocations must be eliminated
