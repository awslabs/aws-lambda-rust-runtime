use proc_macro2::TokenStream;
use quote::quote;
use synstructure::decl_derive;

decl_derive!([LambdaErrorExt, attributes()] => lambda_error_derive);

fn lambda_error_derive(s: synstructure::Structure) -> TokenStream {
    let name = format!("{}", s.ast().ident);

    let err_impl = s.gen_impl(quote! {
        use lambda_runtime_errors::LambdaErrorExt;

        gen impl LambdaErrorExt for @Self {
            fn error_type(&self) -> &str {
                #name
            }
        }
    });

    (quote! {
        #err_impl
    })
}
