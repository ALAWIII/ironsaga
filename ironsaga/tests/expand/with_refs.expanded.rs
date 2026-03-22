use ironsaga::ironcmd;
#[allow(dead_code)]
struct Process<'__ironcmd> {
    data: &str,
    buffer: &mut Vec<u8>,
    rollback_cmd: ::core::option::Option<
        ::std::boxed::Box<dyn ::ironsaga::SyncCommand + '__ironcmd>,
    >,
    result: ::core::option::Option<usize>,
}
impl<'__ironcmd> Process<'__ironcmd> {
    fn new(data: &str, buffer: &mut Vec<u8>) -> Self {
        Self {
            data: data,
            buffer: buffer,
            result: ::core::option::Option::None,
            rollback_cmd: ::core::option::Option::None,
        }
    }
    fn result(&self) -> ::core::option::Option<&usize> {
        self.result.as_ref()
    }
    fn take_result(&mut self) -> ::core::option::Option<usize> {
        self.result.take()
    }
    fn set_rollback(&mut self, rollback: impl ::ironsaga::SyncCommand + '__ironcmd) {
        self.rollback_cmd = ::core::option::Option::Some(
            ::std::boxed::Box::new(rollback),
        );
    }
}
impl<'__ironcmd> ::ironsaga::SyncCommand for Process<'__ironcmd> {
    fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if self.result.is_some() {
            return ::std::result::Result::Ok(());
        }
        let data = &*self.data;
        let buffer = &mut *self.buffer;
        let fire = {
            {
                buffer.push(1);
                data.len()
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
