use proc_macro::TokenStream as TS;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    FnArg, Ident, ItemTrait, Pat, PatIdent, PatType, ReturnType, Type, TypePath, parse_macro_input,
    punctuated::Punctuated,
    token::{Colon, Comma},
};

fn get_export_functions(item: TS) -> TS {
    let input = parse_macro_input!(item as ItemTrait);
    let mut res = quote!();

    for item in input.items.iter().filter_map(|f| match f {
        syn::TraitItem::Fn(f) => Some(f),
        _ => None,
    }) {
        let sig = &item.sig;
        let mut args: Punctuated<_, Comma> = Punctuated::new();
        let macro_sig = {
            let mut sig = sig.clone();
            sig.ident = Ident::new(&format!("plugin_{}", sig.ident), Span::call_site());
            let ReturnType::Type(_, ty) = &mut sig.output else {
                panic!("expected return type!")
            };
            let og = *ty.clone();
            let Type::Path(p) = &mut **ty else {
                panic!("expected PluginResult type")
            };
            *p = syn::parse::<TypePath>(quote!(extism_pdk::FnResult<#og>).into()).unwrap();

            let params = sig
                .inputs
                .into_iter()
                .enumerate()
                .map(|(i, a)| {
                    let FnArg::Typed(mut pat) = a else {
                        panic!("no self types in argument")
                    };
                    let name = Ident::new(&format!("arg{i}"), Span::call_site());
                    pat.pat = Box::new(Pat::Ident(PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: None,
                        ident: name.clone(),
                        subpat: None,
                    }));
                    args.push(name);
                    FnArg::Typed(pat)
                })
                .collect::<Punctuated<_, Comma>>();
            sig.inputs = params;
            sig
        };
        let call = &sig.ident;
        res = quote! {
            #res

            #[extism_pdk::plugin_fn]
            pub #macro_sig {
                Ok($name::#call(#args))
            }
        };
    }

    quote! {
        #[macro_export]
        macro_rules! plugin_gen {
            ($name:ident) => {
                #res
            }
        }
    }
    .into()
}

fn get_plugin_calls(item: TS) -> TS {
    let input = parse_macro_input!(item as ItemTrait);
    let mut res = quote!();

    for item in input.items.iter().filter_map(|f| match f {
        syn::TraitItem::Fn(f) => Some(f),
        _ => None,
    }) {
        let sig = &item.sig;
        let mut args: Punctuated<_, Comma> = Punctuated::new();
        let macro_sig = {
            let mut sig = sig.clone();
            sig.ident = Ident::new(&format!("{}", sig.ident), Span::call_site());
            let ReturnType::Type(rl, mut ty) = sig.output else {
                panic!("expected return type!")
            };
            let copy = *ty.clone();
            let Type::Path(p) = &mut *ty else {
                panic!("expected PluginResult type")
            };
            *p = syn::parse(quote!(Result<#copy, extism::Error>).into()).unwrap();
            sig.output = ReturnType::Type(rl, ty);

            let mut params = sig
                .inputs
                .into_iter()
                .enumerate()
                .map(|(i, a)| {
                    let FnArg::Typed(mut pat) = a else {
                        panic!("no self types in argument")
                    };
                    let name = Ident::new(&format!("arg{i}"), Span::call_site());
                    pat.pat = Box::new(Pat::Ident(PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: None,
                        ident: name.clone(),
                        subpat: None,
                    }));
                    args.push(name);
                    FnArg::Typed(pat)
                })
                .collect::<Punctuated<_, Comma>>();
            params.insert(
                0,
                FnArg::Typed(PatType {
                    attrs: Vec::new(),
                    ty: Box::new(
                        syn::parse_str("&std::sync::Arc<std::sync::Mutex<extism::Plugin>>")
                            .unwrap(),
                    ),
                    pat: Box::new(Pat::Ident(PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: None,
                        ident: Ident::new("__plugin__", Span::call_site()),
                        subpat: None,
                    })),
                    colon_token: Colon {
                        spans: [Span::call_site()],
                    },
                }),
            );
            sig.inputs = params;
            sig
        };
        let args = if args.len() == 1 {
            quote! {#args}
        } else {
            quote! { () }
        };
        let call = format!("plugin_{}", sig.ident);
        res = quote! {
            #res

            pub #macro_sig {
                let res = match __plugin__.lock() {
                    Ok(mut o) => o.call(#call, #args)?,
                    Err(e) => e.into_inner().call(#call, #args)?
                };
                Ok(res)
            }
        };
    }

    quote! {
        #[macro_export]
        macro_rules! plugin_calls {
            () => {
                mod calls {
                    use extism::{*, convert::*};
                    use std::sync::{Arc, Mutex};
                    use super::*;
                    #res
                }
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn plugin(_attr: TS, item: TS) -> TS {
    let res = TokenStream::from(get_export_functions(item.clone()));
    let calls = TokenStream::from(get_plugin_calls(item.clone()));
    let item = TokenStream::from(item);
    let res = quote! {
        #item
        #res
        #calls
    };

    res.into()
}
