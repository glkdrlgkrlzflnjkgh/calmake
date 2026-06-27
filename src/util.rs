pub mod util {
    pub fn comparenums(a:i32, b:i32) -> anyhow::Result<bool> {
        return Ok(a == b);
    }
}