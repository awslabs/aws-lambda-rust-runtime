use proc_macro2::TokenStream;
use quote::quote;
use synstructure::decl_derive;

decl_derive!([LambdaErrorExt, attributes()] => lambda_error_derive);

fn lambda_error_derive(s: synstructure::Structure) -> TokenStream {
    /*
    if let Data::Struct(d) = &s.ast().data {
        if let Fields::Named(fs) = &d.fields {
            for f in fs.named.iter() {
                let field_name = f.ident.clone();
                println!("!!Field name: {}", field_name.expect("Field must be named"));
            }
        }
    }*/

    for v in s.variants() {
        println!("Variant: {}", v.ast().ident);
        /*if let Some(id) = v.prefix {
            println!("Variant: {}", id);
        }*/    
    }

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
