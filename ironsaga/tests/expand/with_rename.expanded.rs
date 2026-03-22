use ironsaga::ironcmd;
#[allow(dead_code)]
struct MyCustomOp<'__ironcmd> {
    value: ::core::option::Option<u32>,
    rollback_cmd: ::core::option::Option<
        ::std::boxed::Box<dyn ::ironsaga::SyncCommand + '__ironcmd>,
    >,
    result: ::core::option::Option<u32>,
}
impl<'__ironcmd> MyCustomOp<'__ironcmd> {
    fn new(value: u32) -> Self {
        Self {
            value: ::core::option::Option::Some(value),
            result: ::core::option::Option::None,
            rollback_cmd: ::core::option::Option::None,
        }
    }
    fn result(&self) -> ::core::option::Option<&u32> {
        self.result.as_ref()
    }
    fn take_result(&mut self) -> ::core::option::Option<u32> {
        self.result.take()
    }
    fn set_rollback(&mut self, rollback: impl ::ironsaga::SyncCommand + '__ironcmd) {
        self.rollback_cmd = ::core::option::Option::Some(
            ::std::boxed::Box::new(rollback),
        );
    }
}
impl<'__ironcmd> ::ironsaga::SyncCommand for MyCustomOp<'__ironcmd> {
    fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if self.result.is_some() {
            return ::std::result::Result::Ok(());
        }
        let value = self.value.take().unwrap();
        let fire = { { value } };
        self.result = ::core::option::Option::Some(fire);
        ::std::result::Result::Ok(())
    }
    fn rollback(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if let ::core::option::Option::Some(r) = self.rollback_cmd.as_mut() {
            r.execute()?;
        }
        ::std::result::Result::Ok(())
    }
}
