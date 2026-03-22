use ironsaga::ironcmd;

#[ironcmd(rename = "Foo", rename = "Bar")]
fn do_thing(x: u32) -> u32 {
    x
}

fn main() {}
