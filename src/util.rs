use crate::Command;
use crate::color;
pub fn comparenums(a: i32, b: i32) -> anyhow::Result<bool> {
    return Ok(a == b);
}
pub fn pretty_cmd(cmd: &Command) -> String {
    let mut s = String::new();

    if let Some(program) = cmd.get_program().to_str() {
        s.push_str(program);
    }

    for arg in cmd.get_args() {
        s.push(' ');
        s.push_str(&arg.to_string_lossy());
    }

    s
}

pub fn impossible_state_bail(message: &str) -> anyhow::Result<()> {
    anyhow::bail!("{}Impossible state reached!!!{} \"{}\" This is a BUG! OPEN AN ISSUE ASAP!!", color::RED, color::RESET, message);
}

