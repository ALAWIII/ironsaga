use ironsaga::ironcmd;
#[allow(dead_code)]
struct MultipleValues<'__ironcmd> {
    url: ::core::option::Option<String>,
    rollback_cmd: ::core::option::Option<
        ::std::boxed::Box<dyn ::ironsaga::AsyncCommand + '__ironcmd>,
    >,
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
            ::std::boxed::Box::new(rollback),
        );
    }
}
impl<'__ironcmd> ::ironsaga::AsyncCommand for MultipleValues<'__ironcmd> {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn execute<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = ::ironsaga::anyhow::Result<()>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                ::ironsaga::anyhow::Result<()>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let mut __self = self;
            let __ret: ::ironsaga::anyhow::Result<()> = {
                if __self.result.is_some() {
                    return ::std::result::Result::Ok(());
                }
                let url = __self.url.take().unwrap();
                let fire = async { { Ok(::alloc::vec::Vec::new()) } };
                __self.result = ::core::option::Option::Some(fire.await);
                __self
                    .result
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
            };
            #[allow(unreachable_code)] __ret
        })
    }
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn rollback<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = ::ironsaga::anyhow::Result<()>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                ::ironsaga::anyhow::Result<()>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let mut __self = self;
            let __ret: ::ironsaga::anyhow::Result<()> = {
                if let ::core::option::Option::Some(rcmd) = __self.rollback_cmd.as_mut()
                {
                    let res = rcmd.execute().await;
                    if let ::std::result::Result::Err(e) = res {
                        let _ = rcmd.rollback().await;
                        return ::std::result::Result::Err(e);
                    }
                }
                ::std::result::Result::Ok(())
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
