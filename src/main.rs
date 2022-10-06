use std::process::Command;
use std::{env, fs};

fn main() {
    // Determine command to run.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Must specify the `fmt` or `check` command");
    }
    let mode = &args[1];

    // Configure formatting options, overwriting only a few of the defaults.
    // https://taplo.tamasfe.dev/.
    let mut taplo_opts = taplo::formatter::Options::default();
    taplo_opts.allowed_blank_lines = 1;
    taplo_opts.indent_entries = true;
    taplo_opts.reorder_keys = true;

    match mode.as_str() {
        "fmt" => fmt(taplo_opts),
        "check" => check(taplo_opts),
        _ => panic!("Unknown command: {mode}"),
    };
}

fn fmt(taplo_opts: taplo::formatter::Options) {
    // Format Solidity with forge
    Command::new("forge")
        .arg("fmt")
        .output()
        .expect("forge fmt failed");

    // Format `foundry.toml` with taplo.
    let config_orig = fs::read_to_string("./foundry.toml").expect("Could not find foundry.toml");
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    fs::write("./foundry.toml", config_fmt).expect("Unable to write foundry.toml");
}

fn check(taplo_opts: taplo::formatter::Options) {
    // Check Solidity with `forge fmt`
    let forge_status = Command::new("forge")
        .arg("fmt")
        .arg("--check")
        .output()
        .expect("forge fmt failed");
    let forge_success = forge_status.status.success();

    // Check TOML with `taplo fmt`
    let config_orig = fs::read_to_string("./foundry.toml").expect("Could not find foundry.toml");
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    let taplo_success = config_orig == config_fmt;

    if !forge_success || !taplo_success {
        eprintln!("Formatting failed! Run `scopelint fmt` to fix");
        std::process::exit(1);
    }
}
