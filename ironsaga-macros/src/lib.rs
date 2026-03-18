use heck::ToPascalCase;
use proc_macro::TokenStream as TS1;
use syn::{LitStr, Token, parse::Parse, parse::ParseStream};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Block, FnArg, Generics, Ident, ImplGenerics, ItemFn, Pat, PatType, ReturnType, Type,
    TypeGenerics, Visibility, WhereClause, parse_macro_input, parse_quote,
};
#[proc_macro_attribute]
pub fn ironcmd(args: TS1, func: TS1) -> TS1 {
    let args = parse_macro_input!(args as IronCmdArgs);
    let func = parse_macro_input!(func as ItemFn);
    let ops = OperationIronStruct::new(func, args);
    let generated = match ops.is_async {
        true => build_async_cmd(&ops),
        false => build_sync_cmd(&ops),
    };
    quote! {#generated}.into()
}

struct IronCmdArgs {
    is_result: bool,
    recursive_rollback: bool,
    rename: Option<String>,
}

impl Parse for IronCmdArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut is_result = false;
        let mut recursive_rollback = false;
        let mut rename = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "result" => is_result = true,
                "recursive_rollback" => recursive_rollback = true,
                "rename" => {
                    input.parse::<Token![=]>()?;
                    let s: LitStr = input.parse()?;
                    rename = Some(s.value());
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

struct OperationIronStruct {
    is_result: bool,
    recursive_rollback: bool,
    s_name: Ident,
    vis: Visibility,
    generics: Generics,
    fn_body: Box<Block>,
    pats_types: Vec<PatType>,
    fields_names: Vec<Pat>,
    fields_types: Vec<Type>,
    ret_type: Type,
    is_async: bool,
}
impl OperationIronStruct {
    pub fn new(func: ItemFn, args: IronCmdArgs) -> Self {
        let sig = func.sig;
        let pats_types = sig
            .inputs
            .into_iter()
            .map(|a| match a {
                FnArg::Typed(t) => t,
                FnArg::Receiver(_) => panic!("self not supported"),
            })
            .collect::<Vec<_>>();
        let (f_names, f_types): (Vec<_>, Vec<_>) = pats_types
            .iter()
            .map(|p| ((*p.pat).clone(), (*p.ty).clone()))
            .collect();
        let ret = match sig.output {
            ReturnType::Type(_, ty) => *ty,
            ReturnType::Default => parse_quote!(()),
        };
        let org_idnt = sig.ident;
        let name_str = args
            .rename
            .unwrap_or_else(|| org_idnt.to_string().to_pascal_case());
        let idnt = Ident::new(&name_str, org_idnt.span());
        Self {
            s_name: idnt, // struct name and Ident
            is_result: args.is_result,
            recursive_rollback: args.recursive_rollback,
            vis: func.vis,
            generics: sig.generics,
            fields_names: f_names,
            fields_types: f_types,
            pats_types,
            fn_body: func.block,
            ret_type: ret,
            is_async: sig.asyncness.is_some(),
        }
    }
    pub fn split_gen_for_impl<'a>(
        &'a self,
    ) -> (ImplGenerics<'a>, TypeGenerics<'a>, Option<&'a WhereClause>) {
        self.generics.split_for_impl()
    }
}
fn generate_async_struct(ops: &OperationIronStruct) -> TokenStream {
    let modified_types = ops.fields_types.iter().map(|t| match is_type_a_ref(t) {
        true => quote! { #t },             // keep &T / &mut T as-is
        false => wrap_type_with_option(t), // wrap owned in Option
    });
    let s_name = &ops.s_name;
    let vis = &ops.vis;
    let gen_params = &ops.generics.params;
    let fields_names = &ops.fields_names;
    let ret_type = &ops.ret_type;
    // no where clause
    // the rollback_cmd might be an async or sync !!
    quote! {
        #vis struct #s_name <'__ironcmd, #gen_params>{
            #(#vis #fields_names: #modified_types,)*
            #vis rollback_cmd: ::core::option::Option<::ironsaga::CommandKind<'__ironcmd>>,
            result: ::core::option::Option<#ret_type>,
        }
    }
}
fn generate_sync_struct(ops: &OperationIronStruct) -> TokenStream {
    let modified_types = ops.fields_types.iter().map(|t| match is_type_a_ref(t) {
        true => quote! { #t },             // keep &T / &mut T as-is
        false => wrap_type_with_option(t), // wrap owned in Option
    });
    let s_name = &ops.s_name;
    let vis = &ops.vis;
    let gen_params = &ops.generics.params;
    let fields_names = &ops.fields_names;
    let ret_type = &ops.ret_type;
    // no where clause
    // the rollback_cmd might be an async or sync !!
    quote! {
        #vis struct #s_name <'__ironcmd, #gen_params>{
            #(#vis #fields_names: #modified_types,)*
            #vis rollback_cmd: ::core::option::Option<::std::boxed::Box<dyn ::ironsaga::SyncCommand + '__ironcmd>>,
            result: ::core::option::Option<#ret_type>,

        }
    }
}

fn derive_async_command(ops: &OperationIronStruct) -> TokenStream {
    let fn_body = &ops.fn_body;
    let pats_types = &ops.pats_types;
    let s_name = &ops.s_name;
    let gen_params = &ops.generics.params;
    let (_, _, where_clause) = ops.split_gen_for_impl();
    let vars = pats_types.iter().map(|p| {
        let name = &p.pat;
        let ty = &p.ty;
        if is_type_a_mut_ref(ty) {
            quote! { let #name= &mut *self.#name } // reborrow as &mut
        } else if is_type_a_ref(ty) {
            quote! { let #name = &*self.#name } // reborrow as &
        } else {
            quote! { let #name = self.#name.take().unwrap() } // consume from Option
        }
    });

    let execute_body = if ops.is_result {
        quote! {
                #(#vars;)*
                let fire = async {#fn_body};
                self.result=::core::option::Option::Some(fire.await);
                self.result
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .map(|_| ())
                    .map_err(|e| ::ironsaga::anyhow::anyhow!("{:?}", e))?;
        }
    } else {
        quote! {
                #(#vars;)*
                let fire = async {#fn_body};
                self.result=::core::option::Option::Some(fire.await);
        }
    };
    let rollback_execution = quote! {
        match rcmd {
             ::ironsaga::CommandKind::SyncCmd(cmd)=> cmd.execute(),
             ::ironsaga::CommandKind::AsyncCmd(cmd)=> cmd.execute().await,
         }
    };
    let rollback_body = if ops.recursive_rollback {
        quote! {
            let rollback_res= #rollback_execution;
            if let ::std::result::Result::Err(e) = rollback_res{
                _ = match rcmd {
                     ::ironsaga::CommandKind::SyncCmd(cmd)=> cmd.rollback(),
                     ::ironsaga::CommandKind::AsyncCmd(cmd)=> cmd.rollback().await,
                 };
                 return ::std::result::Result::Err(e);
            }
        }
    } else {
        quote! {
            #rollback_execution?;
        }
    };
    quote! {
        #[::ironsaga::async_trait::async_trait]
        impl <'__ironcmd,#gen_params> ::ironsaga::AsyncCommand for #s_name <'__ironcmd,#gen_params> #where_clause{
            async fn execute(&mut self)->::ironsaga::anyhow::Result<()>{
                if self.result.is_some(){
                    return ::std::result::Result::Ok(());
                }
                #execute_body
                ::std::result::Result::Ok(())
            }
            async fn rollback(&mut self)->::ironsaga::anyhow::Result<()>{
                if let ::core::option::Option::Some(rcmd)= self.rollback_cmd.as_mut(){
                        #rollback_body
                }
                ::std::result::Result::Ok(())
            }

        }

    }
}

fn derive_sync_command(ops: &OperationIronStruct) -> TokenStream {
    let fn_body = &ops.fn_body;
    let pats_types = &ops.pats_types;
    let s_name = &ops.s_name;
    let gen_params = &ops.generics.params;
    let (_, _, where_clause) = ops.split_gen_for_impl();
    let vars = pats_types.iter().map(|p| {
        let name = &p.pat;
        let ty = &p.ty;
        if is_type_a_mut_ref(ty) {
            quote! { let #name= &mut *self.#name } // reborrow as &mut
        } else if is_type_a_ref(ty) {
            quote! { let #name = &*self.#name } // reborrow as &
        } else {
            quote! { let #name = self.#name.take().unwrap() } // consume from Option
        }
    });

    let execute_body = if ops.is_result {
        quote! {
             #(#vars;)*
             let fire ={#fn_body};
             self.result=::core::option::Option::Some(fire);
             self.result
                 .as_ref()
                 .unwrap()
                 .as_ref()
                 .map(|_| ())
                 .map_err(|e| ::ironsaga::anyhow::anyhow!("{:?}", e))?;
        }
    } else {
        quote! {
            #(#vars;)*
            let fire ={#fn_body};
            self.result=::core::option::Option::Some(fire);
        }
    };
    let rollback_body = if ops.recursive_rollback {
        quote! {
            let rollback_res = r.execute();
            if let ::std::result::Result::Err(e)=rollback_res{
               _ = r.rollback();
                return ::std::result::Result::Err(e);
            }
        }
    } else {
        quote! {
            let rollback_res = r.execute()?;
        }
    };
    quote! {
        impl <'__ironcmd,#gen_params> ::ironsaga::SyncCommand for #s_name <'__ironcmd,#gen_params> #where_clause{
             fn execute(&mut self)->::ironsaga::anyhow::Result<()>{
                 if self.result.is_some(){
                     return ::std::result::Result::Ok(());
                 }
                 #execute_body
                 ::std::result::Result::Ok(())
             }
            fn rollback(&mut self) -> ::ironsaga::anyhow::Result<()> {
                if let ::core::option::Option::Some(r)=self.rollback_cmd.as_mut(){
                    #rollback_body
                }
                ::std::result::Result::Ok(())
            }
        }

    }
}

fn impl_cmd(ops: &OperationIronStruct) -> TokenStream {
    let f_names = &ops.fields_names;
    let wrapped_fields = ops.pats_types.iter().map(|p| {
        let name = &p.pat;
        if is_type_a_ref(&p.ty) {
            quote! {#name}
        } else {
            quote! {#name: ::core::option::Option::Some(#name)}
        }
    });
    let f_types = &ops.fields_types;
    let vis = &ops.vis;
    let s_name = &ops.s_name;
    let ret_type = &ops.ret_type;
    let gen_params = &ops.generics.params;
    let (_, _, where_clause) = ops.split_gen_for_impl();

    let roll_method = if ops.is_async {
        quote! {
            #vis fn set_async_rollback(&mut self,rollback: impl ::ironsaga::AsyncCommand + '__ironcmd){
                self.rollback_cmd= ::core::option::Option::Some(::ironsaga::CommandKind::AsyncCmd(::std::boxed::Box::new(rollback)));
            }
            #vis fn set_sync_rollback(&mut self,rollback: impl ::ironsaga::SyncCommand + '__ironcmd){
                self.rollback_cmd= ::core::option::Option::Some(::ironsaga::CommandKind::SyncCmd(::std::boxed::Box::new(rollback)));
            }
        }
    } else {
        quote! {
            #vis fn set_rollback(&mut self,rollback: impl ::ironsaga::SyncCommand + '__ironcmd){
                self.rollback_cmd= ::core::option::Option::Some(::std::boxed::Box::new(rollback));
            }
        }
    };
    quote! {
        impl <'__ironcmd,#gen_params> #s_name <'__ironcmd,#gen_params> #where_clause {
            #vis fn new(#(#f_names:#f_types,)*)->Self{
                Self{
                    #(#wrapped_fields,)*
                    result: ::core::option::Option::None,
                    rollback_cmd: ::core::option::Option::None,
                }
            }
            #vis fn result(&self)->::core::option::Option<&#ret_type>{
                self.result.as_ref()
            }
            #vis fn take_result(&mut self)-> ::core::option::Option<#ret_type>{
                self.result.take()
            }
            #roll_method
        }
    }
}
fn build_async_cmd(ops: &OperationIronStruct) -> TokenStream {
    let as_s = generate_async_struct(ops);
    let dr_as_cmd = derive_async_command(ops);
    let impl_cmd_s = impl_cmd(ops);
    quote! {
        #as_s
        #impl_cmd_s
        #dr_as_cmd

    }
}
fn build_sync_cmd(ops: &OperationIronStruct) -> TokenStream {
    let as_s = generate_sync_struct(ops);
    let dr_as_cmd = derive_sync_command(ops);
    let impl_cmd_s = impl_cmd(ops);
    quote! {
        #as_s
        #impl_cmd_s
        #dr_as_cmd
    }
}
fn is_type_a_ref(ty: &Type) -> bool {
    matches!(ty, Type::Reference(_))
}
fn wrap_type_with_option(ty: &Type) -> TokenStream {
    quote! {::core::option::Option<#ty>}
}

fn is_type_a_mut_ref(ty: &Type) -> bool {
    if let Type::Reference(r) = ty {
        return r.mutability.is_some();
    }
    false
}
