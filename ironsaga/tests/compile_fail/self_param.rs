use ironsaga::ironcmd;

struct MyOp;
impl MyOp {
    #[ironcmd]
    fn execute(self, x: u32) -> u32 {
        x
    }
}

fn main() {}
