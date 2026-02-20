use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output}

fn build_shell_prompt(mut prompt: String) -> String {
    let user = get_cmd_output_str("whoami", "");
    let mut final_prompt = String::new();
    let curent_dir = std::env::current_dir();
    let mut dir: String = match curent_dir {
        Ok(p) => {
            let os_str = p.as_os_str();
            os_str.to_str().unwrap_or("???").to_string()
        }
        Err(_) => "???".to_string(),
    };
    dir = dir.replace(format!("/home/{}", user.clone()).as_str(), "~");
    prompt.push_str(&dir);
    return format!("{}\nmy_shell > ", prompt);
}

fn print_shell_prompt(mut prompt: String) {
    let prompt = build_shell_prompt(prompt);
    print!("{}", prompt);
    io::stdout().flush().unwrap();
}
fn get_cmd_output_str(cmd: &str, arg: &str) -> String {
    let mut command = Command::new(cmd);
    if !arg.is_empty() {
        command.arg(arg);
    }
    match command.output() {
        Ok(outp) => {
            if outp.status.code().unwrap() == 0 {
                match str::from_utf8(&outp.stdout) {
                    Ok(s) => {
                        return s.trim().to_string();
                    }
                    _ => {}
                };
            }
        }
        _ => {}
    }
    return String::from("???");
}
fn process_command(cmd: &str, arg: &str) -> Result<(), String> {
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
    Ok(())
}
fn main() {
    while true {
        let mut prompt = String::new();
        print_shell_prompt(prompt);
        let mut input_buffer = String::new();
        std::io::stdin().read_line(&mut input_buffer);
        input_buffer = input_buffer.trim().to_string();
        match process_command(&input_buffer, "") {
            Ok(_) => {}
            Err(e) => {
                println!("{e}");
            }
        };
        println!();

    }
    return;
}
