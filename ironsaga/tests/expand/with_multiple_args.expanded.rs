use ironsaga::ironcmd;
#[allow(dead_code)]
struct MultipleValues<'__ironcmd> {
    url: ::core::option::Option<String>,
    rollback_cmd: ::core::option::Option<::ironsaga::CommandKind<'__ironcmd>>,
    result: ::core::option::Option<Result<Vec<u8>, String>>,
}
impl<'__ironcmd> MultipleValues<'__ironcmd> {
    fn new(url: String) -> Self {
        Self {
            url: ::core::option::Option::Some(url),
            result: ::core::option::Option::None,
            rollback_cmd: ::core::option::Option::None,
        }
    }
    fn result(&self) -> ::core::option::Option<&Result<Vec<u8>, String>> {
        self.result.as_ref()
    }
    fn take_result(&mut self) -> ::core::option::Option<Result<Vec<u8>, String>> {
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
impl<'__ironcmd> ::ironsaga::AsyncCommand for MultipleValues<'__ironcmd> {
    async fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if self.result.is_some() {
            return ::std::result::Result::Ok(());
        }
        let url = self.url.take().unwrap();
        let fire = async { { Ok(::alloc::vec::Vec::new()) } };
        self.result = ::core::option::Option::Some(fire.await);
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
    async fn rollback(&mut self) -> ::ironsaga::anyhow::Result<()> {
        if let ::core::option::Option::Some(rcmd) = self.rollback_cmd.as_mut() {
            let res = match rcmd {
                ::ironsaga::CommandKind::SyncCmd(cmd) => cmd.execute(),
                ::ironsaga::CommandKind::AsyncCmd(cmd) => cmd.execute().await,
            };
            if let ::std::result::Result::Err(e) = res {
                let _ = match rcmd {
                    ::ironsaga::CommandKind::SyncCmd(cmd) => cmd.rollback(),
                    ::ironsaga::CommandKind::AsyncCmd(cmd) => cmd.rollback().await,
                };
                return ::std::result::Result::Err(e);
            }
        }
        ::std::result::Result::Ok(())
    }
}
