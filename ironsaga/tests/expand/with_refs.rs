use ironsaga::ironcmd;

#[ironcmd]
fn process(data: &str, buffer: &mut Vec<u8>) -> usize {
    buffer.push(1);
    data.len()
}
