@security
Feature: Template and HTML XSS prevention

  The static site generator must escape user-derived values in templates
  and generated HTML to prevent cross-site scripting attacks.

  Background:
    Given a valid site configuration
    And content files with frontmatter

  Scenario: Template variables are HTML-escaped by default
    Given a content file with title containing HTML special characters
    When the site is built
    Then the generated HTML must contain the escaped title
    And no raw HTML from the title must appear in the output

  Scenario: Template variables with raw suffix preserve HTML
    Given a template using "variable|raw" syntax for intentional HTML
    When the site is built
    Then the variable value must be rendered without escaping

  Scenario: Page titles in list pages are HTML-escaped
    Given a page with a title containing double-quote characters
    When archive or list pages are generated
    Then the title must be properly escaped in HTML attributes

  Scenario: Tag names in tag index are HTML-escaped
    Given a tag name containing angle bracket characters
    When the tags index page is generated
    Then the tag name must be properly escaped in the HTML output

  Scenario: Raw HTML in markdown is sanitized when enabled
    Given a markdown file containing script tags
    And the config has sanitize_html enabled
    When the site is built
    Then script tags must be stripped from the output

  Scenario: Raw HTML in markdown passes through when sanitization disabled
    Given a markdown file containing div tags with class attributes
    And the config has sanitize_html disabled
    When the site is built
    Then the raw HTML must be preserved in the output

  Scenario: Typst source path is not leaked in generated HTML
    Given a Typst content file
    When the site is built
    Then the generated HTML must not contain data-path attributes with source file paths
