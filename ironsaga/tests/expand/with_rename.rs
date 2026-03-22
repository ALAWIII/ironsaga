use ironsaga::ironcmd;

#[ironcmd(rename = "MyCustomOp")]
fn do_thing(value: u32) -> u32 {
    value
}
