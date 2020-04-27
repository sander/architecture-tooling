Feature: Knowledge management

  Scenario: Creating a temporary dataset
    Given I have no dataset
    When I create a temporary dataset
    Then I have a dataset
