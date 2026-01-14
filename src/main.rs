// This file is 100% slopGPT code

use calculator::Compile;
use clap::{Parser, Subcommand};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::fs;
use std::process;

/// calc - A calculator language with multiple execution backends
#[derive(Parser)]
#[command(name = "calc")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input file to execute (legacy mode, use 'calc run <file>' instead)
    #[arg(value_name = "FILE")]
    file: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive REPL with tree-walking interpreter
    Repl,

    /// Start interactive REPL with bytecode VM
    #[cfg(feature = "vm")]
    Vm,

    /// Start interactive REPL with JIT compiler
    #[cfg(feature = "jit")]
    Jit,

    /// Execute a calculator file
    Run {
        /// Path to the calculator file to execute
        #[arg(value_name = "FILE")]
        file: String,

        /// Use bytecode VM for execution
        #[arg(short, long)]
        #[cfg(feature = "vm")]
        vm: bool,

        /// Use JIT compiler for execution
        #[arg(short, long)]
        #[cfg(feature = "jit")]
        jit: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Legacy mode: if a file is provided without subcommand, execute it
    if let Some(file) = cli.file {
        // Use the default engine based on features (like old main.rs)
        cfg_if::cfg_if! {
            if #[cfg(feature = "jit")] {
                run_file::<calculator::Jit>(&file);
            }
            else if #[cfg(feature = "vm")] {
                run_file::<calculator::VM>(&file);
            }
            else {
                run_file::<calculator::Interpreter>(&file);
            }
        }
        return;
    }

    // New subcommand mode
    match cli.command {
        Some(Commands::Repl) => {
            println!("Starting calc REPL (interpreter mode)...");
            println!();
            run_repl::<calculator::Interpreter>();
        }

        #[cfg(feature = "vm")]
        Some(Commands::Vm) => {
            println!("Starting calc REPL (VM mode)...");
            println!();
            run_repl::<calculator::VM>();
        }

        #[cfg(feature = "jit")]
        Some(Commands::Jit) => {
            println!("Starting calc REPL (JIT mode)...");
            println!();
            run_repl::<calculator::Jit>();
        }

        Some(Commands::Run { file, .. }) => {
            #[cfg(all(feature = "vm", feature = "jit"))]
            {
                let use_vm = matches!(&cli.command, Some(Commands::Run { vm: true, .. }));
                let use_jit = matches!(&cli.command, Some(Commands::Run { jit: true, .. }));

                if use_vm && use_jit {
                    eprintln!("Error: Cannot use both --vm and --jit flags");
                    process::exit(1);
                }

                if use_vm {
                    run_file::<calculator::VM>(&file);
                } else if use_jit {
                    run_file::<calculator::Jit>(&file);
                } else {
                    run_file::<calculator::Interpreter>(&file);
                }
            }

            #[cfg(all(feature = "vm", not(feature = "jit")))]
            {
                let use_vm = matches!(&cli.command, Some(Commands::Run { vm: true, .. }));
                if use_vm {
                    run_file::<calculator::VM>(&file);
                } else {
                    run_file::<calculator::Interpreter>(&file);
                }
            }

            #[cfg(all(feature = "jit", not(feature = "vm")))]
            {
                let use_jit = matches!(&cli.command, Some(Commands::Run { jit: true, .. }));
                if use_jit {
                    run_file::<calculator::Jit>(&file);
                } else {
                    run_file::<calculator::Interpreter>(&file);
                }
            }

            #[cfg(not(any(feature = "vm", feature = "jit")))]
            {
                run_file::<calculator::Interpreter>(&file);
            }
        }

        None => {
            // No subcommand and no file - show help
            eprintln!("Error: No command or file provided");
            eprintln!();
            eprintln!("Try 'calc --help' for more information");
            process::exit(1);
        }
    }
}

fn run_repl<T>()
where
    T: Compile,
    T::Output: std::fmt::Debug,
{
    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");

    // Try to load history
    let history_path = home::home_dir().map(|mut p| {
        p.push(".calc_history");
        p
    });

    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    println!("calc REPL - Type 'exit' or press Ctrl+D to quit");
    println!();

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                if line == "exit" || line == "quit" {
                    break;
                }

                let _ = rl.add_history_entry(line);

                let result = T::from_source(line);
                println!("{:?}", result);
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }
}

fn run_file<T>(filename: &str)
where
    T: Compile,
    T::Output: std::fmt::Debug,
{
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let result = T::from_source(&source);
    println!("{:?}", result);
}
