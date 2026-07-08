@correctness
Feature: Build correctness and error handling

  The build pipeline must correctly report errors, handle edge cases,
  and produce accurate output for all content types.

  Background:
    Given a valid site configuration

  Scenario: Build fails when page generation errors occur
    Given content files that cause page generation errors
    When the site is built
    Then the build must return an error
    And the error message must indicate which pages failed

  Scenario: Non-ASCII section slugs do not cause panics
    Given a content file in a section with a Unicode slug
    When the site is built
    Then the build must complete without panicking
    And the generated HTML must contain the correctly capitalized section name

  Scenario: RSS feed paths are correct for default and non-default languages
    Given a multi-language configuration with default language "en"
    When the site is built
    Then the default language RSS feed must exist at the root output directory
    And each non-default language must have its own RSS feed in its language subdirectory

  Scenario: Asset manifest produces valid JSON
    Given static assets to process
    When the asset manifest is generated
    Then the manifest must be valid JSON
    And asset paths with special characters must be properly escaped

  Scenario: Environment variable overrides work for config
    Given a config file and TYPSTIFY environment variables set
    When the CLI loads configuration
    Then the environment variable values must override file values

  Scenario: Asset fingerprints use sufficient hash bits
    Given multiple static assets to fingerprint
    When fingerprints are generated
    Then no two different assets must receive the same fingerprint
    And fingerprints must be at least 12 hex characters long

  Scenario: Dev server debounce does not lose file change events
    Given a running dev server
    When two files change in rapid succession
    Then both changes must be reflected in the rebuild
