//! Proc-macros for Purwa HTTP routing (`#[get]`, `#[resource]`, …).

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::{Attribute, Ident, ItemFn, ItemMod, LitStr, Type};

/// `#[auth(Backend)]` — require a logged-in session for handlers with **no** parameters.
///
/// Redirects to `/login` when unauthenticated. For handlers that need extractors, use
/// [`purwa_auth::CurrentUser`] or [`purwa::auth::CurrentUser`] with the `auth` feature, or add
/// [`purwa::auth::AuthSession`] manually.
///
/// **Requires** crate feature `purwa/auth`.
#[proc_macro_attribute]
pub fn auth(args: TokenStream, input: TokenStream) -> TokenStream {
    auth_impl(args, input)
}

fn auth_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    struct AuthArgs {
        backend: Type,
    }

    impl Parse for AuthArgs {
        fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
            Ok(AuthArgs {
                backend: input.parse()?,
            })
        }
    }

    let AuthArgs { backend } = parse_macro_input!(args as AuthArgs);
    let mut input_fn = parse_macro_input!(input as ItemFn);

    if !input_fn.sig.inputs.is_empty() {
        return syn::Error::new(
            input_fn.sig.inputs.span(),
            "#[auth(Backend)] only supports handlers with no parameters; use CurrentUser<Backend> or AuthSession<Backend>",
        )
        .to_compile_error()
        .into();
    }

    input_fn.sig.output = syn::parse_quote! {
        -> impl ::purwa::axum::response::IntoResponse
    };

    let param: syn::FnArg = syn::parse_quote! {
        mut auth_session: ::purwa::auth::AuthSession<#backend>
    };
    input_fn.sig.inputs.insert(0, param);

    let stmts = &input_fn.block.stmts;
    input_fn.block = syn::parse_quote! {
        {
            use ::purwa::axum::response::IntoResponse;
            if auth_session.user.is_none() {
                return ::purwa::axum::response::Redirect::temporary("/login").into_response();
            }
            let __purwa_body = {
                #(#stmts)*
            };
            __purwa_body.into_response()
        }
    };

    quote! { #input_fn }.into()
}

/// `#[get("/path")] async fn name(...) -> ...`
#[proc_macro_attribute]
pub fn get(args: TokenStream, input: TokenStream) -> TokenStream {
    route_method_macro(
        args,
        input,
        quote! { ::purwa::axum::routing::get },
        quote! { ::purwa::axum::http::Method::GET },
    )
}

/// `#[post("/path")] async fn ...`
#[proc_macro_attribute]
pub fn post(args: TokenStream, input: TokenStream) -> TokenStream {
    route_method_macro(
        args,
        input,
        quote! { ::purwa::axum::routing::post },
        quote! { ::purwa::axum::http::Method::POST },
    )
}

/// `#[put("/path")] async fn ...`
#[proc_macro_attribute]
pub fn put(args: TokenStream, input: TokenStream) -> TokenStream {
    route_method_macro(
        args,
        input,
        quote! { ::purwa::axum::routing::put },
        quote! { ::purwa::axum::http::Method::PUT },
    )
}

/// `#[delete("/path")] async fn ...`
#[proc_macro_attribute]
pub fn delete(args: TokenStream, input: TokenStream) -> TokenStream {
    route_method_macro(
        args,
        input,
        quote! { ::purwa::axum::routing::delete },
        quote! { ::purwa::axum::http::Method::DELETE },
    )
}

fn route_method_macro(
    args: TokenStream,
    input: TokenStream,
    method_router: proc_macro2::TokenStream,
    method_expr: proc_macro2::TokenStream,
) -> TokenStream {
    let path = parse_macro_input!(args as LitStr);
    if !path.value().starts_with('/') {
        return syn::Error::new(path.span(), "route path must start with `/`")
            .to_compile_error()
            .into();
    }

    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = input_fn.sig.ident.clone();
    let install_fn = format_ident!("__purwa_install_{}", fn_name);
    let handler_label = format!(
        "{}::{}",
        std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| "unknown".into()),
        fn_name
    );
    let handler_label_static = LitStr::new(&handler_label, fn_name.span());

    let expanded = quote! {
        #input_fn

        fn #install_fn(
            router: ::purwa::axum::Router,
        ) -> ::purwa::axum::Router {
            router.route(#path, #method_router(#fn_name))
        }

        ::purwa::inventory::submit! {
            ::purwa::routing::RegisteredRoute {
                method: #method_expr,
                path: #path,
                handler_label: #handler_label_static,
                install: ::core::option::Option::Some(#install_fn),
            }
        }
    };

    expanded.into()
}

/// `#[resource("/prefix")] pub mod name { pub async fn index() ... pub async fn destroy() ... }`
///
/// Requires exactly these `pub async fn` names: `index`, `create`, `store`, `show`, `edit`,
/// `update`, `destroy`.
#[proc_macro_attribute]
pub fn resource(args: TokenStream, input: TokenStream) -> TokenStream {
    let prefix_lit = parse_macro_input!(args as LitStr);
    let prefix = prefix_lit.value();
    if !prefix.starts_with('/') {
        return syn::Error::new(
            prefix_lit.span(),
            "resource path prefix must start with `/`",
        )
        .to_compile_error()
        .into();
    }

    let mut module = parse_macro_input!(input as ItemMod);
    module.attrs.retain(|a| !is_purwa_resource_attr(a));

    let mod_ident = module.ident.clone();
    let required = [
        "index", "create", "store", "show", "edit", "update", "destroy",
    ];
    for name in required {
        if !module_has_pub_async_fn(&module, name) {
            return syn::Error::new(
                module.span(),
                format!(
                    "`#[resource]` module `{}` must declare `pub async fn {}`",
                    mod_ident, name
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    let base = prefix.trim_end_matches('/').to_string();
    let path_root = LitStr::new(&base, prefix_lit.span());
    let path_create = LitStr::new(&format!("{base}/create"), prefix_lit.span());
    let path_id = LitStr::new(&format!("{base}/{{id}}"), prefix_lit.span());
    let path_edit = LitStr::new(&format!("{base}/{{id}}/edit"), prefix_lit.span());

    let bundle_root = format_ident!("__purwa_res_{}_bundle_root", mod_ident);
    let bundle_id = format_ident!("__purwa_res_{}_bundle_id", mod_ident);
    let install_create = format_ident!("__purwa_res_{}_create", mod_ident);
    let install_edit = format_ident!("__purwa_res_{}_edit", mod_ident);

    let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| "unknown".into());
    let sp = prefix_lit.span();
    let lbl = |suffix: &str| LitStr::new(&format!("{crate_name}::{mod_ident}::{suffix}"), sp);
    let l_index = lbl("index");
    let l_store = lbl("store");
    let l_create = lbl("create");
    let l_show = lbl("show");
    let l_edit = lbl("edit");
    let l_update = lbl("update");
    let l_destroy = lbl("destroy");

    quote! {
        #module

        fn #bundle_root(router: ::purwa::axum::Router) -> ::purwa::axum::Router {
            router.route(
                #path_root,
                ::purwa::axum::routing::get(#mod_ident::index).post(#mod_ident::store),
            )
        }

        fn #install_create(router: ::purwa::axum::Router) -> ::purwa::axum::Router {
            router.route(#path_create, ::purwa::axum::routing::get(#mod_ident::create))
        }

        fn #bundle_id(router: ::purwa::axum::Router) -> ::purwa::axum::Router {
            router.route(
                #path_id,
                ::purwa::axum::routing::get(#mod_ident::show)
                    .put(#mod_ident::update)
                    .delete(#mod_ident::destroy),
            )
        }

        fn #install_edit(router: ::purwa::axum::Router) -> ::purwa::axum::Router {
            router.route(#path_edit, ::purwa::axum::routing::get(#mod_ident::edit))
        }

        ::purwa::inventory::submit! {
            ::purwa::routing::RegisteredRoute {
                method: ::purwa::axum::http::Method::GET,
                path: #path_root,
                handler_label: #l_index,
                install: ::core::option::Option::Some(#bundle_root),
            }
        }
        ::purwa::inventory::submit! {
            ::purwa::routing::RegisteredRoute {
                method: ::purwa::axum::http::Method::POST,
                path: #path_root,
                handler_label: #l_store,
                install: ::core::option::Option::None,
            }
        }
        ::purwa::inventory::submit! {
            ::purwa::routing::RegisteredRoute {
                method: ::purwa::axum::http::Method::GET,
                path: #path_create,
                handler_label: #l_create,
                install: ::core::option::Option::Some(#install_create),
            }
        }
        ::purwa::inventory::submit! {
            ::purwa::routing::RegisteredRoute {
                method: ::purwa::axum::http::Method::GET,
                path: #path_id,
                handler_label: #l_show,
                install: ::core::option::Option::Some(#bundle_id),
            }
        }
        ::purwa::inventory::submit! {
            ::purwa::routing::RegisteredRoute {
                method: ::purwa::axum::http::Method::GET,
                path: #path_edit,
                handler_label: #l_edit,
                install: ::core::option::Option::Some(#install_edit),
            }
        }
        ::purwa::inventory::submit! {
            ::purwa::routing::RegisteredRoute {
                method: ::purwa::axum::http::Method::PUT,
                path: #path_id,
                handler_label: #l_update,
                install: ::core::option::Option::None,
            }
        }
        ::purwa::inventory::submit! {
            ::purwa::routing::RegisteredRoute {
                method: ::purwa::axum::http::Method::DELETE,
                path: #path_id,
                handler_label: #l_destroy,
                install: ::core::option::Option::None,
            }
        }
    }
    .into()
}

fn is_purwa_resource_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("resource")
}

fn module_has_pub_async_fn(module: &ItemMod, name: &str) -> bool {
    let Some((_, items)) = &module.content else {
        return false;
    };
    let want = Ident::new(name, proc_macro2::Span::call_site());
    for item in items {
        if let syn::Item::Fn(f) = item
            && f.sig.ident == want
        {
            let is_pub = matches!(f.vis, syn::Visibility::Public(_));
            let is_async = f.sig.asyncness.is_some();
            return is_pub && is_async;
        }
    }
    false
}
