use ironsaga::ironcmd;

#[ironcmd(result)]
fn risky_op(input: String) -> Result<String, String> {
    Ok(input)
}
