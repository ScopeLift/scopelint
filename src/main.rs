use std::fs;
use std::process::Command;

fn main() {
    // Format Solidity with forge
    Command::new("forge")
        .arg("fmt")
        .output()
        .expect("forge fmt failed");

    // Format `foundry.toml` with taplo, overwriting only a few of the defaults.
    // https://taplo.tamasfe.dev/
    let mut taplo_opts = taplo::formatter::Options::default();
    taplo_opts.allowed_blank_lines = 1;
    taplo_opts.indent_entries = true;
    taplo_opts.reorder_keys = true;

    let config_path = "./foundry.toml";
    let config_orig = fs::read_to_string(config_path).expect("Could not find foundry.toml");
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    fs::write(config_path, config_fmt).expect("Unable to write foundry.toml");
}
