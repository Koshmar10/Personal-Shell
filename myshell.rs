use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Output};

const WHOAMI_PATH: &str = "/bin/whoami";

fn build_shell_prompt(prompt: &str) -> String {
    return format!("{}\nmy_shell > ", prompt);
}
fn print_shell_prompt(prompt: &str) {
    let prompt = build_shell_prompt(prompt);
    print!("{}", prompt);
    io::stdout().flush().unwrap();
}
fn process_command(cmd: &str, arg: &str) -> Result<String, String> {
    let mut command = Command::new(cmd);
    if !arg.is_empty() {
        command.arg(arg);
    }
    let output = command.output().map_err(|e| e.to_string())?;
    if output.status.code().unwrap() != 0 {
        io::stdout().write_all(&output.stderr);
    } else {
        io::stdout().write_all(&output.stdout);
    }
    return Ok(String::new());
}
fn main() {
    process_command("cat", "sal.txt");
    // print_shell_prompt("~");
    // let mut input_buffer = String::new();
    // std::io::stdin().read_line(&mut input_buffer);
    // println!("{}", &input_buffer);
    // return;
}
