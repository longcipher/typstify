@test
Feature: Test coverage improvements

  Critical code paths must have test coverage, e2e tests must be
  self-contained, and test fixtures must be shared.

  Scenario: E2e tests use bundled test fixtures
    Given e2e tests that silently skip when examples/blog is absent
    When bundled test fixtures are added
    Then the tests must run without external dependencies
    And tests must fail when assertions are wrong

  Scenario: robots.rs has unit tests
    Given robots.rs with zero test coverage
    When unit tests are added
    Then disabled config must produce no file
    And enabled config must produce correct robots.txt format
    And sitemap URL must include base_path

  Scenario: ContentCollector::collect is tested
    Given the collect() method with zero test coverage
    When tests are added with temp directories
    Then the tests must verify page count, section assignments, and taxonomy entries

  Scenario: HtmlGenerator page methods are tested
    Given 7 of 18 HtmlGenerator methods with zero tests
    When tests are added for each method
    Then each test must verify the output contains expected structural elements

  Scenario: Multi-language build is tested
    Given a build pipeline with untested i18n paths
    When a multi-language build test is added
    Then the test must verify output in both language directories

  Scenario: Shared test fixtures replace duplicated helpers
    Given test_config() duplicated across 5 test modules
    When a shared test fixture is created
    Then all test modules must import from the shared location
    And no duplicate test_config() definitions must remain
