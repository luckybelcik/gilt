use std::time::Instant;

use regex::Regex;

use crate::{
    ast::{lowerer::Lowerer, statement::Statement},
    error_handling::diagnostic::{Diagnostic, DiagnosticSeverity},
    semantics::{analyzer::SemanticAnalyzer, symbol_table::SymbolTable, types::GiltType},
    syntax_guard::validate_syntax,
    testing::{subtest_meta::SubtestMetadata, test_expectation::TestExpectation},
};

pub struct Runner {}

type TypedAST = Vec<Statement<GiltType>>;

fn run_pipeline(
    source_code: &str,
) -> Result<
    (
        Result<(), Vec<Diagnostic>>,
        TypedAST,
        Vec<Diagnostic>,
        SymbolTable,
    ),
    (),
> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_gilt::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("Error loading Gilt grammar");

    let tree = parser.parse(&source_code, None).expect("Failed to parse");
    let root_node = tree.root_node();

    let maybe_diagnostics = validate_syntax(&root_node);

    let lowerer = Lowerer::new(&source_code);
    let untyped_ast = match lowerer.lower(root_node) {
        Ok(ast) => ast,
        Err(diags) => {
            for diag in diags {
                println!("Lowering Error: {:?}", diag);
            }
            return Err(());
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    let (typed_ast, _) = analyzer.analyze(untyped_ast);
    Ok((
        maybe_diagnostics,
        typed_ast,
        analyzer.diagnostics,
        analyzer.symbols,
    ))
}

fn find_type(symbol_table: &SymbolTable, var_name: &str) -> Option<GiltType> {
    let symbol = symbol_table.resolve(var_name);
    symbol.map(|symbol| symbol.symbol_type.clone())
}

impl Runner {
    pub fn run_on_files(&self, file_paths: &[&str]) {
        let instant = Instant::now();
        let mut j = 0;
        for path in file_paths {
            let content = std::fs::read_to_string(path)
                .expect(&format!("Could not read test file: {}", path));

            let subtests = content.split("//$ ---");

            for (i, raw_subtest) in subtests.enumerate() {
                let (metadata, code) = self.split_metadata_and_code(raw_subtest);

                let test_id = if metadata.case.is_empty() {
                    format!("{}#{}", path, i + 1)
                } else {
                    format!("{} ({})", path, metadata.case)
                };

                self.execute_test(test_id, metadata, code);

                j += 1;
            }
        }

        let elapsed = instant.elapsed();

        println!("{} tests executed successfully!", j);
        println!(
            "Testing took {} seconds",
            elapsed.as_millis() as f64 / 1000.0
        );
    }

    fn split_metadata_and_code(&self, input: &str) -> (SubtestMetadata, String) {
        let metadata = self.parse_metadata(input);
        let code = input
            .lines()
            .filter(|line| !line.trim().starts_with("//$"))
            .collect::<Vec<_>>()
            .join("\n");
        (metadata, code)
    }

    fn parse_metadata(&self, input: &str) -> SubtestMetadata {
        let mut metadata = SubtestMetadata {
            case: "Unnamed Case".into(),
            expectation: None,
            expected_types: Vec::new(),
            expected_values: Vec::new(),
        };

        let re = Regex::new(
            r"EXPECTED_(?P<type>ERROR|SUCCESS)(?:\s+x\s+(?P<count>\d+))?(?::\s+(?P<payload>.*))?",
        )
        .unwrap();

        let mut case_prefix_count = 0;

        for line in input.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("//$") {
                continue;
            }
            let cmd = trimmed.trim_start_matches("//$").trim();

            if let Some(case) = cmd.strip_prefix("CASE:") {
                metadata.case = case.trim().to_string();
                case_prefix_count += 1;
            } else if let Some(caps) = re.captures(cmd) {
                let count = caps
                    .name("count")
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);

                let payload = caps.name("payload").map(|m| m.as_str().trim().to_string());

                let type_ = caps.name("type").map(|m| m.as_str().trim().to_string());

                let expects_error = if let Some(t) = type_
                    && t == "ERROR"
                {
                    true
                } else {
                    false
                };

                metadata.expectation = Some(TestExpectation {
                    count,
                    payload,
                    expects_error,
                });
            } else if let Some(type_map) = cmd.strip_prefix("EXPECTED_TYPE:") {
                let parts: Vec<&str> = type_map.split("=>").map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    metadata
                        .expected_types
                        .push((parts[0].to_string(), parts[1].to_string()));
                }
            } else if let Some(val_map) = cmd.strip_prefix("EXPECTED_VALUE:") {
                let parts: Vec<&str> = val_map.split("=>").map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    metadata
                        .expected_values
                        .push((parts[0].to_string(), parts[1].to_string()));
                }
            }
        }

        if case_prefix_count > 1 {
            panic!(
                "There shouldn't be two case names in one subtest! Are you forgetting the '//$ ---' separator?"
            )
        }

        metadata
    }

    fn execute_test(&self, test_id: String, metadata: SubtestMetadata, code: String) {
        let (syntax_diagnostics, _, diagnostics, symbol_table) = run_pipeline(&code).unwrap();

        if let Err(syntax_diagnostics) = syntax_diagnostics
            && !syntax_diagnostics.is_empty()
        {
            for diag in syntax_diagnostics {
                panic!("Syntax Error: {:?}", diag.kind);
            }
        }

        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
            .collect();

        if let Some(exp) = &metadata.expectation {
            if exp.expects_error {
                if let Some(expected_name) = &exp.payload {
                    let matches = errors
                        .iter()
                        .filter(|d| d.kind.name() == expected_name)
                        .count();

                    assert_eq!(
                        matches, exp.count as usize,
                        "\n[{}] Error mismatch.\nExpected {} instances of: {}\nFound: {} matches.",
                        test_id, exp.count, expected_name, matches
                    );
                }
            } else {
                if !errors.is_empty() {
                    panic!(
                        "No error expected but {} errors present. Errors: {:?}",
                        errors.len(),
                        errors
                    )
                }
            }
        }

        for (var_name, expected_type_str) in metadata.expected_types {
            let actual_type = find_type(&symbol_table, &var_name).expect(&format!(
                "Variable '{}' not found in AST for test {}",
                var_name, test_id
            ));

            let actual_type_str = format!("{:?}", actual_type);
            assert_eq!(
                actual_type_str.to_lowercase(),
                expected_type_str.to_lowercase(),
                "\n[{}] Type mismatch for '{}'. Expected {}, found {}",
                test_id,
                var_name,
                expected_type_str.to_lowercase(),
                actual_type_str.to_lowercase()
            );
        }
    }
}
