use beetle_cli::{cli, execute_command};

fn main() {
    let command = cli().run();
    let output = execute_command(command);
    println!("{}", output);
}
