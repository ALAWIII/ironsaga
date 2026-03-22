use ironsaga::ironcmd;
#[allow(dead_code)]
struct FetchData<'__ironcmd> {
    url: ::core::option::Option<String>,
    rollback_cmd: ::core::option::Option<::ironsaga::CommandKind<'__ironcmd>>,
    result: ::core::option::Option<Vec<u8>>,
}
impl<'__ironcmd> FetchData<'__ironcmd> {
    fn new(url: String) -> Self {
        Self {
            url: ::core::option::Option::Some(url),
            result: ::core::option::Option::None,
            rollback_cmd: ::core::option::Option::None,
        }
    }
    fn result(&self) -> ::core::option::Option<&Vec<u8>> {
        self.result.as_ref()
    }
    fn take_result(&mut self) -> ::core::option::Option<Vec<u8>> {
        self.result.take()
    }
    fn set_rollback_async(
        &mut self,
        rollback: impl ::ironsaga::AsyncCommand + '__ironcmd,
    ) {
        self.rollback_cmd = ::core::option::Option::Some(
            ::ironsaga::CommandKind::AsyncCmd(::std::boxed::Box::new(rollback)),
        );
    }
    fn set_rollback_sync(
        &mut self,
        rollback: impl ::ironsaga::SyncCommand + '__ironcmd,
    ) {
        self.rollback_cmd = ::core::option::Option::Some(
            ::ironsaga::CommandKind::SyncCmd(::std::boxed::Box::new(rollback)),
        );
    }
}
#[::ironsaga::async_trait::async_trait]
impl<'__ironcmd> ::ironsaga::AsyncCommand for FetchData<'__ironcmd> {
    async fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if self.result.is_some() {
            return ::std::result::Result::Ok(());
        }
        let url = self.url.take().unwrap();
        let fire = async { { ::alloc::vec::Vec::new() } };
        self.result = ::core::option::Option::Some(fire.await);
        ::std::result::Result::Ok(())
    }
    async fn rollback(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if let ::core::option::Option::Some(rcmd) = self.rollback_cmd.as_mut() {
            match rcmd {
                ::ironsaga::CommandKind::SyncCmd(cmd) => cmd.execute(),
                ::ironsaga::CommandKind::AsyncCmd(cmd) => cmd.execute().await,
            }?;
        }
        ::std::result::Result::Ok(())
    }
}
