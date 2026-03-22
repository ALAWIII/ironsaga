use ironsaga::ironcmd;

#[ironcmd(recursive_rollback)]
fn delete_record(id: u64) -> u64 {
    id
}
