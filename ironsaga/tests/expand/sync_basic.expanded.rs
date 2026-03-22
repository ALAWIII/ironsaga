use ironsaga::ironcmd;
#[allow(dead_code)]
struct CreateUser<'__ironcmd> {
    name: ::core::option::Option<String>,
    age: ::core::option::Option<u32>,
    rollback_cmd: ::core::option::Option<
        ::std::boxed::Box<dyn ::ironsaga::SyncCommand + '__ironcmd>,
    >,
    result: ::core::option::Option<String>,
}
impl<'__ironcmd> CreateUser<'__ironcmd> {
    fn new(name: String, age: u32) -> Self {
        Self {
            name: ::core::option::Option::Some(name),
            age: ::core::option::Option::Some(age),
            result: ::core::option::Option::None,
            rollback_cmd: ::core::option::Option::None,
        }
    }
    fn result(&self) -> ::core::option::Option<&String> {
        self.result.as_ref()
    }
    fn take_result(&mut self) -> ::core::option::Option<String> {
        self.result.take()
    }
    fn set_rollback(&mut self, rollback: impl ::ironsaga::SyncCommand + '__ironcmd) {
        self.rollback_cmd = ::core::option::Option::Some(
            ::std::boxed::Box::new(rollback),
        );
    }
}
impl<'__ironcmd> ::ironsaga::SyncCommand for CreateUser<'__ironcmd> {
    fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if self.result.is_some() {
            return ::std::result::Result::Ok(());
        }
        let name = self.name.take().unwrap();
        let age = self.age.take().unwrap();
        let fire = {
            {
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("{0} is {1}", name, age))
                })
            }
        };
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
