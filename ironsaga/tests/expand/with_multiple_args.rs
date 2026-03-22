use ironsaga::ironcmd;
#[ironcmd(result, recursive_rollback, rename = "MultipleValues")]
async fn multiple_args(url: String) -> Result<Vec<u8>, String> {
    Ok(vec![])
}
