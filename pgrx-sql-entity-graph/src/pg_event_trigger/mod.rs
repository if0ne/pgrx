//LICENSE Portions Copyright 2019-2021 ZomboDB, LLC.
//LICENSE
//LICENSE Portions Copyright 2021-2023 Technology Concepts & Design, Inc.
//LICENSE
//LICENSE Portions Copyright 2023-2023 PgCentral Foundation, Inc. <contact@pgcentral.org>
//LICENSE
//LICENSE All rights reserved.
//LICENSE
//LICENSE Use of this source code is governed by the MIT license that can be found in the LICENSE file.

pub mod entity;

use proc_macro2::Span;
use quote::{format_ident, quote};

use crate::{
    enrich::{ToEntityGraphTokens, ToRustCodeTokens},
    finfo::{finfo_v1_extern_c, finfo_v1_tokens},
    CodeEnrichment, ToSqlConfig,
};

#[derive(Debug, Clone)]
pub struct PgEventTrigger {
    func: syn::ItemFn,
    to_sql_config: ToSqlConfig,
}

impl PgEventTrigger {
    pub fn new(func: syn::ItemFn) -> Result<CodeEnrichment<Self>, syn::Error> {
        Ok(CodeEnrichment(Self { func, to_sql_config: Default::default() }))
    }

    fn wrapper_tokens(&self) -> Result<syn::ItemFn, syn::Error> {
        let function_ident = self.func.sig.ident.clone();
        let fcinfo_ident =
            syn::Ident::new("_fcinfo", Span::mixed_site().located_at(function_ident.span()));

        let tokens = quote! {
            fn _internal(fcinfo: ::pgrx::pg_sys::FunctionCallInfo) -> ::pgrx::pg_sys::Datum {
                let fcinfo_ref = unsafe {
                    // SAFETY:  The caller should be Postgres in this case and it will give us a valid "fcinfo" pointer
                    fcinfo.as_ref().expect("fcinfo was NULL from Postgres")
                };
                let result = unsafe { ::pgrx::PgEventTrigger::from_fcinfo(fcinfo_ref) };

                match result {
                    Ok(pg_event_trigger) => {
                        let _ = #function_ident(&pg_event_trigger);
                    }
                    Err(err) => {
                        panic!("PgEventTrigger::from_fcinfo failed: {err}")
                    }
                }

                ::pgrx::pg_sys::Datum::from(0)
            }
            ::pgrx::pg_sys::submodules::panic::pgrx_extern_c_guard(move || _internal(#fcinfo_ident))
        };

        finfo_v1_extern_c(&self.func, fcinfo_ident, tokens)
    }
}

impl ToEntityGraphTokens for PgEventTrigger {
    fn to_entity_graph_tokens(&self) -> proc_macro2::TokenStream {
        let func_sig_ident = &self.func.sig.ident;
        let sql_graph_entity_fn_name =
            format_ident!("__pgrx_internals_event_trigger_{}", func_sig_ident);
        let function_name = func_sig_ident.to_string();
        let to_sql_config = &self.to_sql_config;

        quote! {
            #[no_mangle]
            #[doc(hidden)]
            #[allow(unknown_lints, clippy::no_mangle_with_rust_abi, nonstandard_style)]
            pub extern "Rust" fn #sql_graph_entity_fn_name() -> ::pgrx::pgrx_sql_entity_graph::SqlGraphEntity {
                use core::any::TypeId;
                extern crate alloc;
                use alloc::vec::Vec;
                use alloc::vec;
                let submission = ::pgrx::pgrx_sql_entity_graph::PgEventTriggerEntity {
                    function_name: #function_name,
                    file: file!(),
                    line: line!(),
                    full_path: concat!(module_path!(), "::", stringify!(#func_sig_ident)),
                    module_path: module_path!(),
                    to_sql_config: #to_sql_config,
                };
                ::pgrx::pgrx_sql_entity_graph::SqlGraphEntity::EventTrigger(submission)
            }
        }
    }
}

impl ToRustCodeTokens for PgEventTrigger {
    fn to_rust_code_tokens(&self) -> proc_macro2::TokenStream {
        let wrapper_func = self.wrapper_tokens().expect("Generating wrapper function for trigger");
        let finfo_func = finfo_v1_tokens(wrapper_func.sig.ident.clone()).unwrap();
        let func = &self.func;

        quote! {
            #func
            #wrapper_func
            #finfo_func
        }
    }
}
