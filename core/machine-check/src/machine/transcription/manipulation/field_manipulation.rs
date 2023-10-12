use syn::{
    parse_quote,
    visit_mut::{self, VisitMut},
    Arm, Expr, GenericArgument, Item, ItemImpl, ItemStruct, Lit, PathArguments, Type,
};

pub fn apply(file: &mut syn::File) -> anyhow::Result<()> {
    // add field manipulation to each struct using a visitor
    // visit the file and its submodules and add field manipulation impls
    // for all structs
    struct Visitor(anyhow::Result<()>);
    impl VisitMut for Visitor {
        fn visit_file_mut(&mut self, node: &mut syn::File) {
            let result = apply_to_items(&mut node.items);
            if self.0.is_ok() {
                self.0 = result;
            }
            // delegate to visit nested modules
            visit_mut::visit_file_mut(self, node);
        }

        fn visit_item_mod_mut(&mut self, node: &mut syn::ItemMod) {
            // only modules with bodies have items
            if let Some(content) = &mut node.content {
                let result = apply_to_items(&mut content.1);
                if self.0.is_ok() {
                    self.0 = result;
                }
            }
            // delegate to visit nested modules
            visit_mut::visit_item_mod_mut(self, node);
        }
    }
    let mut visitor = Visitor(Ok(()));
    visitor.visit_file_mut(file);
    visitor.0
}

fn apply_to_items(items: &mut Vec<Item>) -> anyhow::Result<()> {
    // TODO: only apply to items which have corresponding implementations of state or input traits
    let mut impls_to_add = Vec::new();
    for item in items.iter() {
        let Item::Struct(item_struct) = item else {
            continue;
        };
        impls_to_add.extend(create_field_manipulate_impls(item_struct));
    }
    items.extend(impls_to_add.into_iter().map(Item::Impl));
    Ok(())
}

pub fn create_field_manipulate_impls(item_struct: &ItemStruct) -> Vec<ItemImpl> {
    let struct_ident = &item_struct.ident;

    let mut abstract_field_arms = Vec::<Arm>::new();
    let mut abstract_field_mut_arms = Vec::<Arm>::new();
    let mut mark_field_arms = Vec::<Arm>::new();
    let mut mark_field_mut_arms = Vec::<Arm>::new();

    for field in &item_struct.fields {
        let Some(field_ident) = &field.ident else {
            // do not consider unnamed fields
            continue;
        };
        let field_type = &field.ty;

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
        let Some(flavour_segment) = segments_iter.next() else {
            continue;
        };

        let Some(type_segement) = segments_iter.next() else {
            continue;
        };

        if type_segement.ident != "Bitvector" {
            continue;
        }

        let PathArguments::AngleBracketed(arguments) = &type_segement.arguments else {
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
        let field_arm: Arm = parse_quote!(#field_name => Some(self.#field_ident),);
        let field_arm_mut: Arm = parse_quote!(#field_name => Some(&mut self.#field_ident),);

        match flavour_segment.ident.to_string().as_str() {
            "abstr" => {
                abstract_field_arms.push(field_arm);
                abstract_field_mut_arms.push(field_arm_mut);
            }
            "refin" => {
                mark_field_arms.push(field_arm);
                mark_field_mut_arms.push(field_arm_mut);
            }
            _ => (),
        }
    }

    let impl_for_abstract: ItemImpl = parse_quote! {
        impl ::mck::FieldManipulate<::mck::abstr::Bitvector<1>> for #struct_ident  {
            fn get(&self, name: &str) -> ::std::option::Option<::mck::abstr::Bitvector<1>> {
                match name {
                    #(#abstract_field_arms)*
                    _ => ::std::option::Option::None
                }
            }

            fn get_mut(&mut self, name: &str) -> ::std::option::Option<&mut ::mck::abstr::Bitvector<1>> {
                match name {
                    #(#abstract_field_mut_arms)*
                    _ => ::std::option::Option::None
                }
            }
        }
    };
    let impl_for_mark: ItemImpl = parse_quote! {
        impl ::mck::FieldManipulate<::mck::refin::Bitvector<1>> for #struct_ident  {
            fn get(&self, name: &str) -> ::std::option::Option<::mck::refin::Bitvector<1>> {
                match name {
                    #(#mark_field_arms)*
                    _ => ::std::option::Option::None
                }
            }

            fn get_mut(&mut self, name: &str) -> ::std::option::Option<&mut ::mck::refin::Bitvector<1>> {
                match name {
                    #(#mark_field_mut_arms)*
                    _ => ::std::option::Option::None
                }
            }
        }
    };

    vec![impl_for_abstract, impl_for_mark]
}
