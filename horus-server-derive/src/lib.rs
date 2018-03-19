extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(LoggableJob, attributes(LogName))]
pub fn loggable_job(input: TokenStream) -> TokenStream 
{
    let syntax_tree = syn::parse::<DeriveInput>(input).unwrap();
    let gen = impl_loggable_job(&syntax_tree);
    gen.into()
}

fn impl_loggable_job(tree: &syn::DeriveInput) -> quote::Tokens
{
    const LOG_NAME_ATTR: &'static str = "LogName";
    let name = &tree.ident; 
    let mut logname: syn::Ident = syn::Ident::from("none");

    for attr in tree.attrs.iter() {
        let attr = attr.interpret_meta();
        if let Some(a) = attr {
            if let syn::Meta::NameValue(nv) = a {
                if let syn::Lit::Str(v) = nv.lit {
                    if nv.ident == syn::Ident::from(LOG_NAME_ATTR) {
                        logname = syn::Ident::from(v.value());
                    }
                }
            }
        }
    }
    
    if let syn::Data::Struct(_) = tree.data {
        if logname == syn::Ident::from("none") {
            panic!("LoggableJob requires LogName attribute!");
        }

        quote! {
            impl LoggableJob for #name {
                fn log(&mut self, s: &str)
                {
                    self.#logname.push('\n');
                    self.#logname.push_str(s);
                }
            }
        }
    } else {
        panic!("LoggableJob is only implemented for structs!");
    }
}

