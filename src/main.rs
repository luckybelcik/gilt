use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::{
    ast::lowerer::Lowerer, semantics::analyzer::SemanticAnalyzer, testing::runner::Runner,
};

pub mod ast;
pub mod error_handling;
pub mod semantics;
pub mod testing;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: gilt <command> <path>");
        eprintln!("Commands: giltc, gilttest");
        return;
    }

    let command = &args[1];
    let target_path = &args[2];

    match command.as_str() {
        "giltc" => {
            let path = Path::new(target_path);
            if path.extension().and_then(|s| s.to_str()) == Some("gilt") {
                check_file(path);
            } else {
                eprintln!("Error: File must have a .gilt extension!");
            }
        }
        "gilttest" => {
            let path = Path::new(target_path);
            if !path.is_dir() {
                eprintln!("Error: {} is not a directory!", target_path);
                return;
            }

            let test_files = collect_test_files(path);

            if test_files.is_empty() {
                println!("No .gilt-test files found in {}", target_path);
                return;
            }

            println!("Found {} test files. Starting runner...", test_files.len());

            let runner = Runner {};
            let file_refs: Vec<&str> = test_files.iter().map(|p| p.to_str().unwrap()).collect();

            runner.run_on_files(&file_refs);
        }
        _ => eprintln!("Unknown command: {}", command),
    }
}

fn check_file(path: &Path) {
    let source_code = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path.display(), e);
            return;
        }
    };

    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_gilt::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("Error loading Gilt grammar");

    let tree = parser.parse(&source_code, None).expect("Failed to parse");
    let root_node = tree.root_node();

    let lowerer = Lowerer::new(&source_code);
    let untyped_ast = match lowerer.lower(root_node) {
        Ok(ast) => ast,
        Err(diags) => {
            for diag in diags {
                println!("Lowering Error: {:?}", diag);
            }
            return;
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    let (typed_ast, diagnostics) = analyzer.analyze(untyped_ast);

    if diagnostics.is_empty() {
        println!("Analysis successful! Typed AST ready for codegen.");
        for statement in typed_ast {
            println!("{:?}", statement);
        }
    } else {
        for diag in diagnostics {
            println!("Semantic Error: {:?}", diag);
        }
    }
}

fn collect_test_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                let filename = path.to_str().unwrap_or("");
                if filename.ends_with(".gilt-test") {
                    files.push(path);
                }
            }
        }
    }

    files
}
