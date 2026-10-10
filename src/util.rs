use crate::color;

pub fn impossible_state_bail(message: &str) -> anyhow::Result<()> {
    anyhow::bail!("{}Impossible state reached!!!{} \"{}\" This is a BUG! OPEN AN ISSUE ASAP!!", color::RED, color::RESET, message);
}

