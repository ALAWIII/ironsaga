use heck::ToPascalCase;
use proc_macro::TokenStream as TS1;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Block, FnArg, Generics, Ident, ImplGenerics, ItemFn, LitStr, Pat, PatType, ReturnType, Token,
    Type, TypeGenerics, Visibility, WhereClause, parse::Parse, parse::ParseStream,
    parse_macro_input, parse_quote,
};

// ─── Entry Point ─────────────────────────────────────────────────────────────

#[proc_macro_attribute]
pub fn ironcmd(args: TS1, func: TS1) -> TS1 {
    let args = parse_macro_input!(args as IronCmdArgs);
    let func = parse_macro_input!(func as ItemFn);

    let ops = match OperationIronStruct::new(func, args) {
        Ok(ops) => ops,
        Err(e) => return e.to_compile_error().into(),
    };

    if ops.is_async {
        build_async_cmd(&ops)
    } else {
        build_sync_cmd(&ops)
    }
    .into()
}

// ─── Args Parsing ─────────────────────────────────────────────────────────────

struct IronCmdArgs {
    is_result: bool,
    recursive_rollback: bool,
    rename: Option<String>,
}

impl Parse for IronCmdArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut is_result = false;
        let mut recursive_rollback = false;
        let mut rename: Option<String> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "result" => {
                    if is_result {
                        return Err(syn::Error::new(ident.span(), "duplicate argument `result`"));
                    }
                    is_result = true;
                }
                "recursive_rollback" => {
                    if recursive_rollback {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate argument `recursive_rollback`",
                        ));
                    }
                    recursive_rollback = true;
                }
                "rename" => {
                    if rename.is_some() {
                        return Err(syn::Error::new(ident.span(), "duplicate argument `rename`"));
                    }
                    input.parse::<Token![=]>()?;
                    rename = Some(input.parse::<LitStr>()?.value());
                }
                unknown => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown ironcmd argument `{unknown}`"),
                    ));
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(IronCmdArgs {
            is_result,
            recursive_rollback,
            rename,
        })
    }
}

// ─── Operation Context ────────────────────────────────────────────────────────

struct OperationIronStruct {
    is_result: bool,
    recursive_rollback: bool,
    s_name: Ident,
    vis: Visibility,
    generics: Generics,
    fn_body: Box<Block>,
    pats_types: Vec<PatType>, // original fn args (mut preserved for new() sig)
    field_names: Vec<Pat>,    // mut-stripped for struct fields
    field_types: Vec<Type>,
    ret_type: Type,
    is_async: bool,
}

impl OperationIronStruct {
    pub fn new(func: ItemFn, args: IronCmdArgs) -> syn::Result<Self> {
        let sig = func.sig;

        let mut pats_types = Vec::new();
        for arg in sig.inputs {
            match arg {
                FnArg::Typed(t) => pats_types.push(t),
                FnArg::Receiver(r) => {
                    return Err(syn::Error::new_spanned(
                        r.self_token,
                        "`self` is not supported in #[ironcmd] functions",
                    ));
                }
            }
        }

        // Strip `mut` only for struct field names; keep original for fn args
        let (field_names, field_types): (Vec<_>, Vec<_>) = pats_types
            .iter()
            .map(|p| {
                let mut pat = (*p.pat).clone();
                if let Pat::Ident(ref mut pi) = pat {
                    pi.mutability = None;
                }
                (pat, (*p.ty).clone())
            })
            .unzip();

        let ret_type = match sig.output {
            ReturnType::Type(_, ty) => *ty,
            ReturnType::Default => parse_quote!(()),
        };

        let name_str = args
            .rename
            .unwrap_or_else(|| sig.ident.to_string().to_pascal_case());
        let s_name = Ident::new(&name_str, sig.ident.span());

        Ok(Self {
            is_result: args.is_result,
            recursive_rollback: args.recursive_rollback,
            s_name,
            vis: func.vis,
            generics: sig.generics,
            fn_body: func.block,
            pats_types,
            field_names,
            field_types,
            ret_type,
            is_async: sig.asyncness.is_some(),
        })
    }

    fn split_for_impl(&self) -> (ImplGenerics<'_>, TypeGenerics<'_>, Option<&WhereClause>) {
        self.generics.split_for_impl()
    }
}

// ─── Struct Generation ────────────────────────────────────────────────────────

fn generate_struct(ops: &OperationIronStruct, rollback_type: TokenStream) -> TokenStream {
    let field_types = ops.field_types.iter().map(|t| {
        if is_type_a_ref(t) {
            quote! { #t }
        } else {
            quote! { ::core::option::Option<#t> }
        }
    });
    let OperationIronStruct {
        s_name,
        vis,
        field_names,
        ret_type,
        generics,
        ..
    } = ops;
    let gen_params = &generics.params;

    quote! {
        #[allow(dead_code)]
        #vis struct #s_name<'__ironcmd, #gen_params> {
            #(#vis #field_names: #field_types,)*
            #vis rollback_cmd: ::core::option::Option<#rollback_type>,
            result: ::core::option::Option<#ret_type>,
        }
    }
}

fn generate_async_struct(ops: &OperationIronStruct) -> TokenStream {
    generate_struct(ops, quote! { ::ironsaga::CommandKind<'__ironcmd> })
}

fn generate_sync_struct(ops: &OperationIronStruct) -> TokenStream {
    generate_struct(
        ops,
        quote! { ::std::boxed::Box<dyn ::ironsaga::SyncCommand + '__ironcmd> },
    )
}

// ─── Impl Block ───────────────────────────────────────────────────────────────

fn impl_cmd(ops: &OperationIronStruct) -> TokenStream {
    let OperationIronStruct {
        vis,
        s_name,
        ret_type,
        generics,
        pats_types,
        ..
    } = ops;
    let gen_params = &generics.params;
    let (_, _, where_clause) = ops.split_for_impl();

    // fn new() args preserve original mut bindings
    let fn_args = pats_types.iter().map(|p| {
        let pat = &p.pat;
        let ty = &p.ty;
        quote! { #pat: #ty }
    });

    // struct init: owned → Some(val), refs → as-is
    let init_fields = pats_types.iter().map(|p| {
        let ty = &p.ty;
        let mut stripped = (*p.pat).clone();
        if let Pat::Ident(ref mut pi) = stripped {
            pi.mutability = None;
        }
        if is_type_a_ref(ty) {
            quote! { #stripped: #stripped }
        } else {
            quote! { #stripped: ::core::option::Option::Some(#stripped) }
        }
    });

    let rollback_setter = if ops.is_async {
        quote! {
            #vis fn set_rollback_async(&mut self, rollback: impl ::ironsaga::AsyncCommand + '__ironcmd) {
                self.rollback_cmd = ::core::option::Option::Some(
                    ::ironsaga::CommandKind::AsyncCmd(::std::boxed::Box::new(rollback))
                );
            }
            #vis fn set_rollback_sync(&mut self, rollback: impl ::ironsaga::SyncCommand + '__ironcmd) {
                self.rollback_cmd = ::core::option::Option::Some(
                    ::ironsaga::CommandKind::SyncCmd(::std::boxed::Box::new(rollback))
                );
            }
        }
    } else {
        quote! {
            #vis fn set_rollback(&mut self, rollback: impl ::ironsaga::SyncCommand + '__ironcmd) {
                self.rollback_cmd = ::core::option::Option::Some(::std::boxed::Box::new(rollback));
            }
        }
    };

    quote! {
        impl<'__ironcmd, #gen_params> #s_name<'__ironcmd, #gen_params> #where_clause {
            #vis fn new(#(#fn_args,)*) -> Self {
                Self {
                    #(#init_fields,)*
                    result: ::core::option::Option::None,
                    rollback_cmd: ::core::option::Option::None,
                }
            }

            #vis fn result(&self) -> ::core::option::Option<&#ret_type> {
                self.result.as_ref()
            }

            #vis fn take_result(&mut self) -> ::core::option::Option<#ret_type> {
                self.result.take()
            }

            #rollback_setter
        }
    }
}

// ─── Shared Execute Body ──────────────────────────────────────────────────────

fn build_vars(pats_types: &[PatType]) -> impl Iterator<Item = TokenStream> + '_ {
    pats_types.iter().map(|p| {
        let ty = &p.ty;
        let orig_pat = &p.pat; // preserves mut for let binding
        let mut stripped = (*p.pat).clone();
        if let Pat::Ident(ref mut pi) = stripped {
            pi.mutability = None;
        }
        if is_type_a_mut_ref(ty) {
            quote! { let #stripped = &mut *self.#stripped }
        } else if is_type_a_ref(ty) {
            quote! { let #stripped = &*self.#stripped }
        } else {
            quote! { let #orig_pat = self.#stripped.take().unwrap() }
        }
    })
}

fn build_execute_body(ops: &OperationIronStruct, is_async: bool) -> TokenStream {
    let fn_body = &ops.fn_body;
    let vars = build_vars(&ops.pats_types);

    let fire = if is_async {
        quote! {
            let fire = async { #fn_body };
            self.result = ::core::option::Option::Some(fire.await);
        }
    } else {
        quote! {
            let fire = { #fn_body };
            self.result = ::core::option::Option::Some(fire);
        }
    };

    let result_check = if ops.is_result {
        quote! {
            self.result
                .as_ref()
                .unwrap()
                .as_ref()
                .map(|_| ())
                .map_err(|e| ::ironsaga::anyhow::anyhow!("{:?}", e))?;
        }
    } else {
        quote! {}
    };

    quote! {
        #(#vars;)*
        #fire
        #result_check
    }
}

// ─── Trait Impls ──────────────────────────────────────────────────────────────

fn derive_async_command(ops: &OperationIronStruct) -> TokenStream {
    let s_name = &ops.s_name;
    let gen_params = &ops.generics.params;
    let (_, _, where_clause) = ops.split_for_impl();
    let exec_body = build_execute_body(ops, true);

    let rollback_body = if ops.recursive_rollback {
        quote! {
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
        }
    } else {
        quote! {
            if let ::core::option::Option::Some(rcmd) = self.rollback_cmd.as_mut() {
                match rcmd {
                    ::ironsaga::CommandKind::SyncCmd(cmd) => cmd.execute(),
                    ::ironsaga::CommandKind::AsyncCmd(cmd) => cmd.execute().await,
                }?;
            }
        }
    };

    quote! {
        #[::ironsaga::async_trait::async_trait]
        impl<'__ironcmd, #gen_params> ::ironsaga::AsyncCommand
            for #s_name<'__ironcmd, #gen_params> #where_clause
        {
            async fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
                if self.result.is_some() {
                    return ::std::result::Result::Ok(());
                }
                #exec_body
                ::std::result::Result::Ok(())
            }

            async fn rollback(&mut self) -> ::ironsaga::anyhow::Result<()> {
                #rollback_body
                ::std::result::Result::Ok(())
            }
        }
    }
}

fn derive_sync_command(ops: &OperationIronStruct) -> TokenStream {
    let s_name = &ops.s_name;
    let gen_params = &ops.generics.params;
    let (_, _, where_clause) = ops.split_for_impl();
    let exec_body = build_execute_body(ops, false);

    let rollback_body = if ops.recursive_rollback {
        quote! {
            if let ::core::option::Option::Some(r) = self.rollback_cmd.as_mut() {
                let res = r.execute();
                if let ::std::result::Result::Err(e) = res {
                    let _ = r.rollback(); // best-effort: preserve original error
                    return ::std::result::Result::Err(e);
                }
            }
        }
    } else {
        quote! {
            if let ::core::option::Option::Some(r) = self.rollback_cmd.as_mut() {
                r.execute()?;
            }
        }
    };

    quote! {
        impl<'__ironcmd, #gen_params> ::ironsaga::SyncCommand
            for #s_name<'__ironcmd, #gen_params> #where_clause
        {
            fn execute(&mut self) -> ::ironsaga::anyhow::Result<()> {
                if self.result.is_some() {
                    return ::std::result::Result::Ok(());
                }
                #exec_body
                ::std::result::Result::Ok(())
            }

            fn rollback(&mut self) -> ::ironsaga::anyhow::Result<()> {
                #rollback_body
                ::std::result::Result::Ok(())
            }
        }
    }
}

// ─── Builders ─────────────────────────────────────────────────────────────────

fn build_async_cmd(ops: &OperationIronStruct) -> TokenStream {
    let s = generate_async_struct(ops);
    let i = impl_cmd(ops);
    let t = derive_async_command(ops);
    quote! { #s #i #t }
}

fn build_sync_cmd(ops: &OperationIronStruct) -> TokenStream {
    let s = generate_sync_struct(ops);
    let i = impl_cmd(ops);
    let t = derive_sync_command(ops);
    quote! { #s #i #t }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn is_type_a_ref(ty: &Type) -> bool {
    matches!(ty, Type::Reference(_))
}

fn is_type_a_mut_ref(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r) if r.mutability.is_some())
}
