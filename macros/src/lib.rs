use convert_case::{Case, Casing};
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
        named.named.push(
            syn::Field::parse_named
                .parse2(syn::parse_quote! {
                    /// The alpha component of the color.
                    pub alpha: Component
                })
                .unwrap(),
        );

        named.named.push(
            syn::Field::parse_named
                .parse2(syn::parse_quote! {
                    /// Holds various flags about the color.
                    pub flags: crate::Flags
                })
                .unwrap(),
        );

        named.named.push(
            syn::Field::parse_named
                .parse2(syn::parse_quote! {
                    /// A placeholder for the color space.
                    _space: crate::color::SpacePlaceholder
                })
                .unwrap(),
        );

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
                let field_name = format!("_{}", ident.to_string().to_case(Case::Snake));
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
                #field1: impl Into<crate::ComponentDetails>,
                #field2: impl Into<crate::ComponentDetails>,
                #field3: impl Into<crate::ComponentDetails>,
                alpha: impl Into<crate::ComponentDetails>
            ) -> Self {
                let mut flags = crate::Flags::empty();

                let #field1 = #field1
                    .into()
                    .value_and_flag(&mut flags, crate::Flags::C0_IS_NONE);
                let #field2 = #field2
                    .into()
                    .value_and_flag(&mut flags, crate::Flags::C1_IS_NONE);
                let #field3 = #field3
                    .into()
                    .value_and_flag(&mut flags, crate::Flags::C2_IS_NONE);
                let alpha = alpha
                    .into()
                    .value_and_flag(&mut flags, crate::Flags::ALPHA_IS_NONE);

                Self {
                    #field1,
                    #field2,
                    #field3,
                    alpha,
                    flags,
                    _space: Default::default(),
                    #(#phantom_fields: std::marker::PhantomData,)*
                }
            }
        }
    };

    // TODO: Generate tests to check if the model has the same layout as Color.

    quote! {
        #input
        #new_impl
    }
    .into()
}
