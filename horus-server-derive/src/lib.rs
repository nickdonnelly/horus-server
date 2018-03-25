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

#[proc_macro_derive(FromInt)]
pub fn from_int(input: TokenStream) -> TokenStream
{
    let syntax_tree = syn::parse::<DeriveInput>(input).unwrap();
    let gen = impl_from_int(&syntax_tree);
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

                fn logs(&self) -> String
                {
                    self.#logname.clone()
                }
            }
        }
    } else {
        panic!("LoggableJob is only implemented for structs!");
    }
}

fn impl_from_int(tree: &syn::DeriveInput) -> quote::Tokens
{
    if let syn::Data::Enum(ref _enum) = tree.data {
        let name = &tree.ident;
        let variants = &_enum.variants; 

        let match_i32 = match_i32(&name, variants);
        
        quote! {
            impl FromInt for #name {
                fn from_int(i: i32) -> Self
                {
                    match i {
                        #(#match_i32)*
                        _ => unreachable!()
                    }
                }
            }
        }
        
    } else {
        panic!("FromInt is only implemented for enums!");
    }
}

fn match_i32(
    name: &syn::Ident, 
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) 
    -> Vec<quote::Tokens>
{
    use quote::ToTokens;

    let mut tokens: Vec<quote::Tokens> = Vec::new();

    for (_index, variant) in variants.iter().enumerate() {
        let ident = &variant.ident;
        match &variant.discriminant {
            &Some((_, syn::Expr::Lit(ref lit))) => {
                match lit.lit {
                    syn::Lit::Int(ref lint) => {
                        let lint_tokens = lint.into_tokens();
                        let t = quote!{
                            #lint_tokens => #name::#ident,
                        };
                        tokens.push(t);
                    },
                    _ => panic!("Enum variants must be literal integers!")
                }
            },
            _ => panic!("Enum variants must be literal integers!")
        }
    }

    tokens
}
