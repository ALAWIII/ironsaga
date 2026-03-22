use ironsaga::ironcmd;

#[ironcmd(rename)]
fn do_thing(x: u32) -> u32 {
    x
}

fn main() {}
