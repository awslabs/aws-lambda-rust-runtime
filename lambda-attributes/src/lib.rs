extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, AttributeArgs, FnArg, ItemFn, Meta, NestedMeta};

/// Return true if attribute macro args declares http flavor in the form `#[lambda(http)]`
fn is_http(args: &AttributeArgs) -> bool {
    args.iter().any(|arg| match arg {
        NestedMeta::Meta(Meta::Path(path)) => path.is_ident("http"),
        _ => false,
    })
}

#[proc_macro_attribute]
pub fn lambda(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemFn);
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let asyncness = &input.sig.asyncness;
    let inputs = &input.sig.inputs;

    if name != "main" {
        let tokens = quote_spanned! { name.span() =>
            compile_error!("only the main function can be tagged with #[lambda]");
        };
        return TokenStream::from(tokens);
    }

    if asyncness.is_none() {
        let tokens = quote_spanned! { input.span() =>
          compile_error!("the async keyword is missing from the function declaration");
        };
        return TokenStream::from(tokens);
    }

    let result = match inputs.len() {
        2 => {
            let event = match inputs.first().expect("expected event argument") {
                FnArg::Typed(arg) => arg,
                _ => {
                    let tokens = quote_spanned! { inputs.span() =>
                        compile_error!("fn main's first argument must be fully formed");
                    };
                    return TokenStream::from(tokens);
                }
            };
            let event_name = &event.pat;
            let event_type = &event.ty;
            let context = match inputs.iter().nth(1).expect("expected context argument") {
                FnArg::Typed(arg) => arg,
                _ => {
                    let tokens = quote_spanned! { inputs.span() =>
                        compile_error!("fn main's second argument must be fully formed");
                    };
                    return TokenStream::from(tokens);
                }
            };
            let context_name = &context.pat;
            let context_type = &context.ty;

            if is_http(&args) {
                quote_spanned! { input.span() =>

                    #(#attrs)*
                    #asyncness fn main() {
                        async fn actual(#event_name: #event_type, #context_name: #context_type) #ret #body

                        let f = lambda_http::handler(actual);
                        lambda_http::lambda::run(f).await.unwrap();
                    }
                }
            } else {
                quote_spanned! { input.span() =>

                    #(#attrs)*
                    #asyncness fn main() {
                        async fn actual(#event_name: #event_type, #context_name: #context_type) #ret #body

                        let f = lambda::handler_fn(actual);
                        lambda::run(f).await.unwrap();
                    }
                }
            }
        }
        _ => {
            let tokens = quote_spanned! { inputs.span() =>
                compile_error!("The #[lambda] macro can expects two arguments: a triggered event and lambda context.");
            };
            return TokenStream::from(tokens);
        }
    };

    result.into()
}
