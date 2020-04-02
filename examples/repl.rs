use rhai::{Engine, EvalAltResult, Position, Scope, AST};

#[cfg(not(feature = "no_optimize"))]
use rhai::OptimizationLevel;

use std::{
    io::{stdin, stdout, Write},
    iter,
};

fn print_error(input: &str, err: EvalAltResult) {
    let lines: Vec<_> = input.trim().split('\n').collect();

    let line_no = if lines.len() > 1 {
        match err.position() {
            p if p.is_none() => "".to_string(),
            p if p.is_eof() => format!("{}: ", lines.len()),
            p => format!("{}: ", p.line().unwrap()),
        }
    } else {
        "".to_string()
    };

    // Print error
    let pos = err.position();
    let pos_text = format!(" ({})", pos);

    let pos = if pos.is_eof() {
        let last = lines[lines.len() - 1];
        Position::new(lines.len(), last.len() + 1)
    } else {
        pos
    };

    match pos {
        p if p.is_eof() => panic!("should not be EOF"),
        p if p.is_none() => {
            // No position
            println!("{}", err);
        }
        p => {
            // Specific position
            println!("{}{}", line_no, lines[p.line().unwrap() - 1]);

            let err_text = match err {
                EvalAltResult::ErrorRuntime(err, _) if !err.is_empty() => {
                    format!("Runtime error: {}", err)
                }
                err => err.to_string(),
            };

            println!(
                "{0:>1$} {2}",
                "^",
                line_no.len() + p.position().unwrap(),
                err_text.replace(&pos_text, "")
            );
        }
    }
}

fn print_help() {
    println!("help       => print this help");
    println!("quit, exit => quit");
    println!("ast        => print the last AST");
    println!("astu       => print the last raw, un-optimized AST");
    println!(r"end a line with '\' to continue to the next line.");
    println!();
}

fn main() {
    let mut engine = Engine::new();

    #[cfg(not(feature = "no_optimize"))]
    engine.set_optimization_level(OptimizationLevel::None);

    let mut scope = Scope::new();

    let mut input = String::new();
    let mut ast_u: Option<AST> = None;
    let mut ast: Option<AST> = None;

    println!("Rhai REPL tool");
    println!("==============");
    print_help();

    loop {
        print!("rhai> ");
        stdout().flush().expect("couldn't flush stdout");

        input.clear();

        loop {
            if let Err(err) = stdin().read_line(&mut input) {
                panic!("input error: {}", err);
            }

            let line = input.as_str().trim_end();

            // Allow line continuation
            if line.ends_with('\\') {
                let len = line.len();
                input.truncate(len - 1);
                input.push('\n');
            } else {
                break;
            }

            print!("> ");
            stdout().flush().expect("couldn't flush stdout");
        }

        let script = input.trim();

        // Implement standard commands
        match script {
            "help" => {
                print_help();
                continue;
            }
            "exit" | "quit" => break, // quit
            "astu" => {
                if matches!(&ast_u, Some(_)) {
                    // print the last un-optimized AST
                    println!("{:#?}", ast_u.as_ref().unwrap());
                } else {
                    println!("()");
                }
                continue;
            }
            "ast" => {
                if matches!(&ast, Some(_)) {
                    // print the last AST
                    println!("{:#?}", ast.as_ref().unwrap());
                } else {
                    println!("()");
                }
                continue;
            }
            _ => (),
        }

        if let Err(err) = engine
            .compile_with_scope(&scope, &script)
            .map_err(EvalAltResult::ErrorParsing)
            .and_then(|r| {
                ast_u = Some(r);

                #[cfg(not(feature = "no_optimize"))]
                {
                    engine.set_optimization_level(OptimizationLevel::Full);
                    ast = Some(engine.optimize_ast(&scope, ast_u.as_ref().unwrap()));
                    engine.set_optimization_level(OptimizationLevel::None);
                }

                #[cfg(feature = "no_optimize")]
                {
                    ast = ast_u.clone();
                }

                engine
                    .consume_ast_with_scope(&mut scope, true, ast.as_ref().unwrap())
                    .or_else(|err| match err {
                        EvalAltResult::Return(_, _) => Ok(()),
                        err => Err(err),
                    })
            })
        {
            println!();
            print_error(&input, err);
            println!();
        }
    }
}
