use ironsaga::ironcmd;

#[ironcmd]
fn create_user(name: String, age: u32) -> String {
    format!("{name} is {age}")
}
