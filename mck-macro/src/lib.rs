use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, GenericArgument, Lit, PathArguments, Type};

#[proc_macro_derive(FieldManipulate)]
pub fn field_manipulate(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let derive_input = parse_macro_input!(input as DeriveInput);
    let Data::Struct(data_struct) = derive_input.data else {
        panic!("Struct expected")
    };

    let derive_ident = derive_input.ident;

    let mut abstract_field_arms = Vec::<proc_macro2::TokenStream>::new();
    let mut mark_field_arms = Vec::<proc_macro2::TokenStream>::new();

    for field in data_struct.fields {
        let field_ident = field.ident.unwrap();
        let field_type = field.ty;

        let Type::Path(path_type) = field_type else {
            continue;
        };
        if path_type.qself.is_some() || path_type.path.leading_colon.is_none() {
            continue;
        }
        let mut segments_iter = path_type.path.segments.iter();
        let Some(crate_segment) = segments_iter.next() else {
            continue;
        };
        if crate_segment.ident != "mck" {
            continue;
        }
        let Some(type_segment) = segments_iter.next() else {
            continue;
        };

        let PathArguments::AngleBracketed(arguments) = &type_segment.arguments else {
            continue;
        };

        if arguments.args.len() != 1 {
            continue;
        }
        let GenericArgument::Const(Expr::Lit(expr_lit)) = &arguments.args[0] else {
            continue;
        };
        let Lit::Int(lit_int) = &expr_lit.lit else {
            continue;
        };

        let Ok(lit_val) = lit_int.base10_parse::<u32>() else {
            continue;
        };
        if lit_val != 1 {
            continue;
        }

        if segments_iter.next().is_some() {
            continue;
        }

        let field_name = field_ident.to_string();
        let field_arm = quote!(#field_name => Some(self.#field_ident),);

        match type_segment.ident.to_string().as_str() {
            "ThreeValuedBitvector" => abstract_field_arms.push(field_arm),
            "MarkBitvector" => mark_field_arms.push(field_arm),
            _ => (),
        }
    }

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl ::mck::FieldManipulate<::mck::ThreeValuedBitvector<1>> for #derive_ident  {
            fn get(&self, name: &str) -> ::std::option::Option<::mck::ThreeValuedBitvector<1>> {
                match name {
                    #(#abstract_field_arms)*
                    _ => ::std::option::Option::None
                }
            }

            fn get_mut(&mut self, name: &str) -> ::std::option::Option<&mut ::mck::ThreeValuedBitvector<1>> {
                match name {
                    _ => ::std::option::Option::None
                }
            }
        }

        impl ::mck::FieldManipulate<::mck::MarkBitvector<1>> for #derive_ident  {
            fn get(&self, name: &str) -> ::std::option::Option<::mck::MarkBitvector<1>> {
                match name {
                    #(#mark_field_arms)*
                    _ => ::std::option::Option::None
                }
            }

            fn get_mut(&mut self, name: &str) -> ::std::option::Option<&mut ::mck::MarkBitvector<1>> {
                match name {
                    _ => ::std::option::Option::None
                }
            }
        }

    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
