use colored::Colorize;
use kari_move::{
    base::{
        build::Build,
        coverage::{Coverage, CoverageSummaryOptions},
        disassemble::Disassemble,
        docgen::Docgen,
        errmap::Errmap,
        info::Info,
        new::New,
        prove::Prove,
        test::Test,
    },
    run_cli, Command, Move,
};
use move_core_types::errmap::ErrorMapping;
use move_package::BuildConfig;
use move_vm_test_utils::gas_schedule::zero_cost_schedule;
use std::{path::PathBuf, process::exit};

struct CommandInfo {
    name: &'static str,
    description: &'static str,
}

const COMMANDS: &[CommandInfo] = &[
    CommandInfo { name: "build", description: "Build the package" },
    CommandInfo { name: "coverage", description: "Inspect test coverage for this package. A previous test run with the `--coverage` flag must have" },
    CommandInfo { name: "disassemble", description: "Disassemble Move bytecode" },
    CommandInfo { name: "docgen", description: "Generate documentation" },
    CommandInfo { name: "errmap", description: "Generate error map" },
    CommandInfo { name: "info", description: "Print address information" },
    CommandInfo { name: "new", description: "Create a new Move package with name `name` at `path`. If `path` is not provided the package will" },
    CommandInfo { name: "prove", description: "Prove a Move module" },
    CommandInfo { name: "test", description: "Run Move unit tests" },
];

fn display_help(show_error: bool) {
    if show_error {
        println!("\n{}", "ERROR: Invalid command".red().bold());
    }

    println!("{}", "USAGE:".bright_yellow().bold());
    println!("kari move <command> [options]\n");

    println!("{}", "COMMANDS:".bright_yellow().bold());

    let max_name_len = COMMANDS.iter().map(|cmd| cmd.name.len()).max().unwrap_or(0);

    for cmd in COMMANDS {
        println!(
            "  {}{}  {}",
            cmd.name.green().bold(),
            " ".repeat(max_name_len - cmd.name.len() + 2),
            cmd.description.bright_white()
        );
    }
    println!();

    exit(1);
}

pub fn handle_move_command() {
    let args: Vec<String> = std::env::args().collect();
    let cost_table = zero_cost_schedule();
    let error_mapping = ErrorMapping::default();

    // Check for minimum arguments
    if args.len() <= 2 {
        display_help(false);
        return;
    }

    let move_args = Move {
        package_path: None,
        verbose: false,
        build_config: BuildConfig::default(),
    };

    let cmd = match args.get(2).map(|s| s.as_str()) {
        Some("build") => Command::Build(Build {}),
        Some("coverage") => Command::Coverage(Coverage {
            options: CoverageSummaryOptions::Summary {
                functions: false,
                output_csv: false,
            },
        }),
        Some("disassemble") => Command::Disassemble(Disassemble {
            interactive: false,
            package_name: None,
            module_or_script_name: String::new(),
        }),
        Some("docgen") => Command::Docgen(Docgen {
            section_level_start: Some(0),
            exclude_private_fun: false,
            exclude_specs: false,
            independent_specs: false,
            exclude_impl: false,
            toc_depth: Some(3),
            no_collapsed_sections: false,
            output_directory: None,
            compile_relative_to_output_dir: false,
            references_file: None,
            template: Vec::new(),
            include_dep_diagrams: false,
            include_call_diagrams: false,
        }),
        Some("errmap") => Command::Errmap(Errmap {
            error_prefix: None,
            output_file: PathBuf::new(),
        }),
        Some("info") => Command::Info(Info {}),

        Some("new") => match args.get(3).map(String::from) {
            Some(name) if !name.is_empty() => Command::New(New { name }),
            _ => {
                eprintln!("Error: Project name is required. Usage: kari move new <project_name>");
                std::process::exit(1);
            }
        },

        Some("test") => Command::Test(Test {
            gas_limit: None,
            filter: None,
            list: false,
            num_threads: 8,
            report_statistics: false,
            report_storage_on_error: false,
            ignore_compile_warnings: false,
            check_stackless_vm: false,
            verbose_mode: false,
            compute_coverage: false,
        }),
        Some("prove") => Command::Prove(Prove {
            target_filter: None,
            for_test: false,
            options: None,
        }),
        _ => {
            display_help(true);
            return;
        }
    };

    if let Err(e) = run_cli(Vec::new(), &cost_table, &error_mapping, move_args, cmd) {
        println!("\n{}: {}", "ERROR".red().bold(), e);
        exit(1);
    }
}
