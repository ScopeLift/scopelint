use std::process::Command;

fn main() {
    // Format solidity with forge
    Command::new("forge")
        .arg("fmt")
        .output()
        .expect("forge fmt failed");
}
