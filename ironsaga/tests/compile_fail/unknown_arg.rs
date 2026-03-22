use ironsaga::ironcmd;

#[ironcmd(foobar)]
fn do_thing(x: u32) -> u32 {
    x
}

fn main() {}
