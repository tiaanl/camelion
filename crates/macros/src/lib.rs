use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Parser;

#[proc_macro]
pub fn gen_model(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::ItemStruct);

    if input.fields.len() != 3 {
        return quote! {
            compile_error!("Models must have exactly 3 fields, one for each component of the color.")
        }
        .into();
    }

    let field_names = input
        .fields
        .iter()
        .map(|f| f.ident.clone())
        .collect::<Vec<_>>();
    debug_assert!(field_names.len() == 3);

    let field1 = &field_names[0];
    let field2 = &field_names[1];
    let field3 = &field_names[2];

    // Make sure the 3 specified fields are public.
    input.fields.iter_mut().for_each(|f| {
        f.vis = syn::Visibility::Public(Default::default());
    });

    // Add some derives.
    // TODO: Check if the derives are already there.
    let attr = syn::Attribute::parse_outer
        .parse2(syn::parse_quote! {
            #[derive(Clone, Debug)]
        })
        .unwrap();
    input.attrs.extend(attr);

    let mut phantom_fields: Vec<syn::Ident> = vec![];

    if let syn::Fields::Named(ref mut named) = input.fields {
        // Phantom Fields.
        input
            .generics
            .params
            .iter()
            .map(|g| {
                if let syn::GenericParam::Type(type_param) = g {
                    type_param.ident.clone()
                } else {
                    panic!("unsupported generic type")
                }
            })
            .for_each(|ident| {
                let field_name = format!("_{}", ident.to_string().to_lowercase());
                let field_name = syn::Ident::new(field_name.as_str(), Span::call_site());
                phantom_fields.push(field_name.clone());

                named.named.push(
                    syn::Field::parse_named
                        .parse2(syn::parse_quote! {
                            #field_name: std::marker::PhantomData<#ident>
                        })
                        .unwrap(),
                );
            });
    }

    let struct_name = input.ident.clone();
    let (impl_gen, type_gen, _) = input.generics.split_for_impl();

    let new_impl = quote! {
        impl #impl_gen #struct_name #type_gen {
            /// Create a new color having this color space.
            pub fn new(
                #field1: crate::color::Component,
                #field2: crate::color::Component,
                #field3: crate::color::Component,
            ) -> Self {
                Self {
                    #field1,
                    #field2,
                    #field3,
                    #(#phantom_fields: std::marker::PhantomData,)*
                }
            }

            /// Convert this model into generic components.
            pub fn to_components(&self) -> crate::color::Components {
                crate::color::Components(self.#field1, self.#field2, self.#field3)
            }
        }

        impl #impl_gen From<crate::color::Components> for #struct_name #type_gen {
            fn from(value: crate::color::Components) -> Self {
                Self::new(value.0, value.1, value.2)
            }
        }

        impl #impl_gen crate::models::Model for #struct_name #type_gen
        where
            Self: crate::color::CssColorSpaceId
        {
            fn to_color(&self, alpha: Option<crate::color::Component>) -> crate::color::Color {
                crate::color::Color::new(
                    <Self as crate::color::CssColorSpaceId>::ID,
                    if self.#field1.is_nan() { None } else { Some(self.#field1) },
                    if self.#field2.is_nan() { None } else { Some(self.#field2) },
                    if self.#field3.is_nan() { None } else { Some(self.#field3) },
                    alpha
                )
            }
        }
    };

    quote! {
        #input
        #new_impl
    }
    .into()
}
