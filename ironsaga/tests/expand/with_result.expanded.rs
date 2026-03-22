use ironsaga::ironcmd;
#[allow(dead_code)]
struct RiskyOp<'__ironcmd> {
    input: ::core::option::Option<String>,
    rollback_cmd: ::core::option::Option<
        ::std::boxed::Box<dyn ::ironsaga::SyncCommand + '__ironcmd>,
    >,
    result: ::core::option::Option<Result<String, String>>,
}
impl<'__ironcmd> RiskyOp<'__ironcmd> {
    fn new(input: String) -> Self {
        Self {
            input: ::core::option::Option::Some(input),
            result: ::core::option::Option::None,
            rollback_cmd: ::core::option::Option::None,
        }
    }
    fn result(&self) -> ::core::option::Option<&Result<String, String>> {
        self.result.as_ref()
    }
    fn take_result(&mut self) -> ::core::option::Option<Result<String, String>> {
        self.result.take()
    }
    fn set_rollback(&mut self, rollback: impl ::ironsaga::SyncCommand + '__ironcmd) {
        self.rollback_cmd = ::core::option::Option::Some(
            ::std::boxed::Box::new(rollback),
        );
    }
}
impl<'__ironcmd> ::ironsaga::SyncCommand for RiskyOp<'__ironcmd> {
    fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if self.result.is_some() {
            return ::std::result::Result::Ok(());
        }
        let input = self.input.take().unwrap();
        let fire = { { Ok(input) } };
        self.result = ::core::option::Option::Some(fire);
        self.result
            .as_ref()
            .unwrap()
            .as_ref()
            .map(|_| ())
            .map_err(|e| ::anyhow::Error::msg(
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("{0:?}", e))
                }),
            ))?;
        ::std::result::Result::Ok(())
    }
    fn rollback(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if let ::core::option::Option::Some(r) = self.rollback_cmd.as_mut() {
            r.execute()?;
        }
        ::std::result::Result::Ok(())
    }
}
