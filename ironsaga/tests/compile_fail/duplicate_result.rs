use ironsaga::ironcmd;

#[ironcmd(result, result)]
fn do_thing(x: u32) -> Result<u32, String> {
    Ok(x)
}

fn main() {}
