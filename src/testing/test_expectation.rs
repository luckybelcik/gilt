pub struct TestExpectation {
    pub count: u32,
    pub payload: Option<String>,
    pub expects_error: bool,
}
