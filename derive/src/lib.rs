use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_derive(Validate, attributes(avast))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    expand_derive_validate(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Expand derive validate.
fn expand_derive_validate(input: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    // Parse for context type.
    let context_type = parse_context_type(input)?;
    // Generate the body.
    let body = generate_body(input)?;
    Ok(quote! {
        impl #impl_generics ::avast::Validate for #ident #ty_generics #where_clause {
            type Context = #context_type;
            fn validate(&self, ctx: &Self::Context) -> ::avast::Validity {
                let mut all = ::avast::Validity::valid();
                #body
                all
            }
        }
    })
}

fn parse_context_type(input: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let mut context_type = quote!(());
    for attr in input.attrs.iter() {
        if attr.path().is_ident("avast") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("ctx") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let parsed: syn::Type = content.parse()?;
                    context_type = quote!(#parsed);
                }
                Ok(())
            })?;
        }
    }
    Ok(context_type)
}

fn generate_body(input: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    match &input.data {
        syn::Data::Struct(data) => generate_body_struct(data),
        _ => Err(syn::Error::new(input.span(), "Unsupported data type")),
    }
}

fn generate_body_struct(data: &syn::DataStruct) -> syn::Result<TokenStream2> {
    let mut validations = Vec::new();
    for field in data.fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        for attr in field.attrs.iter() {
            if attr.path().is_ident("avast") {
                attr.parse_nested_meta(|meta| {
                    let validator_name = meta.path.get_ident().unwrap();
                    let args = match meta.input.parse::<syn::Expr>() {
                        Ok(expr) => {
                            match expr {
                                syn::Expr::Tuple(tuple) => {
                                    let args = tuple.elems;
                                    quote!(, #args)
                                },
                                _ => return Err(syn::Error::new(expr.span(), "Cannot parse validator args")),
                            }
                        },
                        Err(_) => quote!(),
                    };
                    validations.push(quote! {
                        all.combine(
                            #validator_name(&self.#field_name #args)
                                .push_blacklist(stringify!(#field_name), stringify!(#validator_name))
                        );
                    });
                    Ok(())
                })?;
            }
        }
    }
    Ok(quote! {
        #( #validations )*
    })
}