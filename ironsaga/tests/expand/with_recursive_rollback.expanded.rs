use ironsaga::ironcmd;
#[allow(dead_code)]
struct DeleteRecord<'__ironcmd> {
    id: ::core::option::Option<u64>,
    rollback_cmd: ::core::option::Option<
        ::std::boxed::Box<dyn ::ironsaga::SyncCommand + '__ironcmd>,
    >,
    result: ::core::option::Option<u64>,
}
impl<'__ironcmd> DeleteRecord<'__ironcmd> {
    fn new(id: u64) -> Self {
        Self {
            id: ::core::option::Option::Some(id),
            result: ::core::option::Option::None,
            rollback_cmd: ::core::option::Option::None,
        }
    }
    fn result(&self) -> ::core::option::Option<&u64> {
        self.result.as_ref()
    }
    fn take_result(&mut self) -> ::core::option::Option<u64> {
        self.result.take()
    }
    fn set_rollback(&mut self, rollback: impl ::ironsaga::SyncCommand + '__ironcmd) {
        self.rollback_cmd = ::core::option::Option::Some(
            ::std::boxed::Box::new(rollback),
        );
    }
}
impl<'__ironcmd> ::ironsaga::SyncCommand for DeleteRecord<'__ironcmd> {
    fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if self.result.is_some() {
            return ::std::result::Result::Ok(());
        }
        let id = self.id.take().unwrap();
        let fire = { { id } };
        self.result = ::core::option::Option::Some(fire);
        ::std::result::Result::Ok(())
    }
    fn rollback(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if let ::core::option::Option::Some(r) = self.rollback_cmd.as_mut() {
            let res = r.execute();
            if let ::std::result::Result::Err(e) = res {
                let _ = r.rollback();
                return ::std::result::Result::Err(e);
            }
        }
        ::std::result::Result::Ok(())
    }
}
