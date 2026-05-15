use crate::testing::test_expectation::TestExpectation;

pub struct SubtestMetadata {
    pub case: String,
    pub expectations: Vec<TestExpectation>,
    pub expected_types: Vec<(String, String)>,
    pub expected_values: Vec<(String, String)>,
    pub tags: Vec<String>,
}
