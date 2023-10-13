use proc_macro2::Span;
use syn::{
    parse_quote, punctuated::Punctuated, token::Brace, BinOp, Block, Expr, ExprBinary, ExprPath,
    ExprReference, ExprStruct, FnArg, Generics, Ident, ImplItem, ImplItemFn, ImplItemType, Item,
    ItemImpl, ItemMod, ItemStruct, Pat, PatType, Path, PathArguments, PathSegment, Receiver,
    ReturnType, Signature, Stmt, Type, TypeReference, Visibility,
};
use syn_path::path;

use crate::machine::transcription::util::{
    create_expr_call, create_expr_field, create_expr_path, create_field_value, create_ident,
    create_pat_ident, create_path_from_ident, create_path_from_name, create_type_path,
    path_rule::{self, PathRule, PathRuleSegment},
};

use self::{refin_fn::transcribe_item_impl, refin_stmt::create_join_stmt};

mod refin_fn;
mod refin_stmt;

pub fn apply(file: &mut syn::File) -> anyhow::Result<()> {
    // the mark will be in a new module under the abstract

    // create items to add to the module
    let mut mark_file_items = Vec::<Item>::new();
    for item in &file.items {
        match item {
            Item::Struct(s) => {
                apply_transcribed_item_struct(&mut mark_file_items, s)?;

                if s.ident == "Input" || s.ident == "State" {
                    let s_ident = &s.ident;
                    let decay_fn = generate_force_decay_fn(s)?;
                    let decay_trait: Path = parse_quote!(::mck::refin::Decay<super::#s_ident>);
                    mark_file_items.push(Item::Impl(ItemImpl {
                        attrs: vec![],
                        defaultness: None,
                        unsafety: None,
                        impl_token: Default::default(),
                        generics: Generics::default(),
                        trait_: Some((None, decay_trait, Default::default())),
                        self_ty: Box::new(Type::Path(create_type_path(create_path_from_ident(
                            s.ident.clone(),
                        )))),
                        brace_token: Default::default(),
                        items: vec![ImplItem::Fn(decay_fn)],
                    }))
                }
            }
            Item::Impl(i) => {
                let mut transcribed = transcribe_item_impl(i)?;
                if let Some(trait_) = &mut transcribed.trait_ {
                    path_rule::apply_to_path(&mut trait_.1, mark_path_rules())?;
                }

                if let Some((None, trait_path, _)) = &i.trait_ {
                    if trait_path.leading_colon.is_some() {
                        let mut iter = trait_path.segments.iter();
                        if let Some(crate_seg) = iter.next() {
                            if let Some(flavour_seg) = iter.next() {
                                if let Some(type_seg) = iter.next() {
                                    if crate_seg.ident == "mck"
                                        && flavour_seg.ident == "abstr"
                                        && (type_seg.ident == "Input"
                                            || type_seg.ident == "State"
                                            || type_seg.ident == "Machine")
                                    {
                                        let s_ty = i.self_ty.as_ref();
                                        // add abstract type
                                        transcribed.items.push(ImplItem::Type(ImplItemType {
                                            attrs: vec![],
                                            vis: Visibility::Inherited,
                                            defaultness: None,
                                            type_token: Default::default(),
                                            ident: create_ident("Abstract"),
                                            generics: Generics::default(),
                                            eq_token: Default::default(),
                                            ty: Type::Path(create_type_path(
                                                parse_quote!(super::#s_ty),
                                            )),
                                            semi_token: Default::default(),
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }

                mark_file_items.push(Item::Impl(transcribed));
            }
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        };
    }
    // create new module at the end of the file that will contain the mark

    let mod_mark = Item::Mod(ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Public(Default::default()),
        unsafety: None,
        mod_token: Default::default(),
        ident: Ident::new("refin", Span::call_site()),
        content: Some((Brace::default(), mark_file_items)),
        semi: None,
    });
    file.items.push(mod_mark);
    Ok(())
}

fn apply_transcribed_item_struct(items: &mut Vec<Item>, s: &ItemStruct) -> anyhow::Result<()> {
    // apply path rules and push struct
    let mut s = s.clone();
    path_rule::apply_to_item_struct(&mut s, mark_path_rules())?;
    let ident_string = s.ident.to_string();

    // TODO: add the implementations only for state and input according to traits
    if ident_string.as_str() != "Machine" {
        let join_impl = generate_join_impl(&s)?;
        let mark_single_impl = generate_mark_single_impl(&s)?;
        let fabricator_impl = generate_fabricator_impl(&s)?;
        let markable_impl = generate_markable_impl(&s)?;
        // add struct
        items.push(Item::Struct(s));
        // add implementations
        items.push(Item::Impl(join_impl));
        items.push(Item::Impl(mark_single_impl));
        items.push(Item::Impl(fabricator_impl));
        items.push(Item::Impl(markable_impl));
    } else {
        // add struct
        items.push(Item::Struct(s));
    }

    Ok(())
}

fn generate_join_impl(s: &ItemStruct) -> anyhow::Result<ItemImpl> {
    let struct_type = Type::Path(create_type_path(Path::from(s.ident.clone())));
    let join_impl_trait = (None, path!(::mck::refin::Join), Default::default());
    let self_type = Type::Path(create_type_path(path!(Self)));
    let self_input = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: Some(Default::default()),
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: Some(Default::default()),
            elem: Box::new(self_type.clone()),
        })),
    });
    let other_ident = create_ident("other");
    let other_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(other_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(self_type),
    });

    let mut join_stmts = Vec::new();
    for (index, field) in s.fields.iter().enumerate() {
        let self_expr_path = create_expr_path(path!(self));
        let other_expr_path = create_expr_path(create_path_from_ident(other_ident.clone()));

        let left = Expr::Field(create_expr_field(Expr::Path(self_expr_path), index, field));
        let right = Expr::Field(create_expr_field(Expr::Path(other_expr_path), index, field));
        let join_stmt = create_join_stmt(left, right);
        join_stmts.push(join_stmt);
    }

    let join_fn = ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("apply_join"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input, other_input]),
            variadic: None,
            output: ReturnType::Default,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: join_stmts,
        },
    };

    Ok(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Generics::default(),
        trait_: Some(join_impl_trait),
        self_ty: Box::new(struct_type),
        brace_token: Default::default(),
        items: vec![ImplItem::Fn(join_fn)],
    })
}

fn generate_force_decay_fn(state_struct: &ItemStruct) -> anyhow::Result<ImplItemFn> {
    let mark_type = Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: None,
        mutability: None,
        elem: Box::new(Type::Path(create_type_path(path!(Self)))),
    });
    let s_ident = &state_struct.ident;
    let abstract_type = Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: None,
        mutability: Some(Default::default()),
        elem: Box::new(Type::Path(create_type_path(parse_quote!(super::#s_ident)))),
    });

    let decay_input = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: None,
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(mark_type),
    });
    let target_ident = create_ident("target");
    let target_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(target_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(abstract_type),
    });

    let mut stmts = Vec::new();
    for (index, field) in state_struct.fields.iter().enumerate() {
        let decay_expr_path = create_expr_path(path!(self));
        let target_expr_path = create_expr_path(create_path_from_ident(target_ident.clone()));

        let decay_field = Expr::Field(create_expr_field(Expr::Path(decay_expr_path), index, field));
        let decay_ref = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: None,
            expr: Box::new(decay_field),
        });
        let target_field = Expr::Field(create_expr_field(
            Expr::Path(target_expr_path),
            index,
            field,
        ));
        let target_ref = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: Some(Default::default()),
            expr: Box::new(target_field),
        });
        let stmt = Stmt::Expr(
            Expr::Call(create_expr_call(
                Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: path!(::mck::refin::Decay::force_decay),
                }),
                Punctuated::from_iter(vec![decay_ref, target_ref]),
            )),
            Some(Default::default()),
        );
        stmts.push(stmt);
    }

    Ok(ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("force_decay"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![decay_input, target_input]),
            variadic: None,
            output: ReturnType::Default,
        },
        block: Block {
            brace_token: Default::default(),
            stmts,
        },
    })
}

fn generate_mark_single_impl(s: &ItemStruct) -> anyhow::Result<ItemImpl> {
    let struct_type = Type::Path(create_type_path(Path::from(s.ident.clone())));
    let mark_single_trait = (None, path!(::mck::refin::MarkSingle), Default::default());
    let self_type = Type::Path(create_type_path(path!(Self)));
    let self_input = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: Some(Default::default()),
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: Some(Default::default()),
            elem: Box::new(self_type.clone()),
        })),
    });
    let offer_ident = create_ident("offer");
    let offer_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(offer_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: None,
            elem: Box::new(self_type),
        })),
    });

    let mut result_expr: Option<Expr> = None;
    for (index, field) in s.fields.iter().enumerate() {
        let self_expr_path = create_expr_path(path!(self));
        let other_expr_path = create_expr_path(create_path_from_ident(offer_ident.clone()));

        let left = Expr::Field(create_expr_field(Expr::Path(self_expr_path), index, field));
        let left = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: Some(Default::default()),
            expr: Box::new(left),
        });
        let right = Expr::Field(create_expr_field(Expr::Path(other_expr_path), index, field));
        let right = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: None,
            expr: Box::new(right),
        });

        let func_expr = Expr::Path(create_expr_path(path!(
            ::mck::refin::MarkSingle::apply_single_mark
        )));
        let expr = Expr::Call(create_expr_call(
            func_expr,
            Punctuated::from_iter(vec![left, right]),
        ));

        if let Some(previous_expr) = result_expr.take() {
            // short-circuiting or for simplicity
            result_expr = Some(Expr::Binary(ExprBinary {
                attrs: vec![],
                left: Box::new(previous_expr),
                op: BinOp::Or(Default::default()),
                right: Box::new(expr),
            }))
        } else {
            result_expr = Some(expr);
        }
    }

    // if there are no fields, return false
    let result_expr = result_expr.unwrap_or(Expr::Path(create_expr_path(path!(false))));

    let return_type = ReturnType::Type(
        Default::default(),
        Box::new(Type::Path(create_type_path(path!(bool)))),
    );

    let apply_fn = ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("apply_single_mark"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input, offer_input]),
            variadic: None,
            output: return_type,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(result_expr, None)],
        },
    };

    Ok(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Generics::default(),
        trait_: Some(mark_single_trait),
        self_ty: Box::new(struct_type),
        brace_token: Default::default(),
        items: vec![ImplItem::Fn(apply_fn)],
    })
}

fn fabricate_first_fn(s: &ItemStruct, self_input: FnArg) -> ImplItemFn {
    let s_ident = s.ident.clone();
    let fabricated_type: Path = parse_quote!(super::#s_ident);
    let return_type = ReturnType::Type(
        Default::default(),
        Box::new(Type::Path(create_type_path(fabricated_type.clone()))),
    );

    let mut struct_expr_fields = Punctuated::new();

    for (index, field) in s.fields.iter().enumerate() {
        let self_field_expr = Expr::Field(create_expr_field(
            Expr::Path(create_expr_path(path!(self))),
            index,
            field,
        ));
        let self_ref_expr = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: None,
            expr: Box::new(self_field_expr),
        });

        let init_expr = Expr::Call(create_expr_call(
            Expr::Path(create_expr_path(path!(::mck::misc::Meta::proto_first))),
            Punctuated::from_iter(vec![self_ref_expr]),
        ));

        let field_value = create_field_value(index, field, init_expr);

        struct_expr_fields.push(field_value);
    }

    let struct_expr = Expr::Struct(ExprStruct {
        attrs: vec![],
        qself: None,
        path: fabricated_type,
        brace_token: Default::default(),
        fields: struct_expr_fields,
        dot2_token: None,
        rest: None,
    });

    ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("proto_first"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input]),
            variadic: None,
            output: return_type,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(struct_expr, Default::default())],
        },
    }
}

fn increment_fabricated_fn(s: &ItemStruct, self_input: FnArg) -> ImplItemFn {
    let fabricated_ident = create_ident("proto");
    let s_ident = s.ident.clone();
    let fabricated_type: Path = parse_quote!(super::#s_ident);
    let fabricated_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(fabricated_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: None,
            mutability: Some(Default::default()),
            elem: Box::new(Type::Path(create_type_path(fabricated_type))),
        })),
    });

    let return_type = ReturnType::Type(
        Default::default(),
        Box::new(Type::Path(create_type_path(path!(bool)))),
    );

    let mut result_expr = None;

    for (index, field) in s.fields.iter().enumerate() {
        let self_expr_path = create_expr_path(path!(self));
        let fabricated_expr_path =
            create_expr_path(create_path_from_ident(fabricated_ident.clone()));

        let self_expr = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: None,
            expr: Box::new(Expr::Field(create_expr_field(
                Expr::Path(self_expr_path),
                index,
                field,
            ))),
        });
        let fabricated_expr = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: Some(Default::default()),
            expr: Box::new(Expr::Field(create_expr_field(
                Expr::Path(fabricated_expr_path),
                index,
                field,
            ))),
        });
        let func_expr = Expr::Path(create_expr_path(path!(::mck::misc::Meta::proto_increment)));
        let expr = Expr::Call(create_expr_call(
            func_expr,
            Punctuated::from_iter(vec![self_expr, fabricated_expr]),
        ));
        if let Some(previous_expr) = result_expr.take() {
            // short-circuiting or for simplicity
            result_expr = Some(Expr::Binary(ExprBinary {
                attrs: vec![],
                left: Box::new(previous_expr),
                op: BinOp::Or(Default::default()),
                right: Box::new(expr),
            }))
        } else {
            result_expr = Some(expr);
        }
    }

    // if there are no fields, return false
    let result_expr = result_expr.unwrap_or(Expr::Path(create_expr_path(path!(false))));

    ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("proto_increment"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input, fabricated_input]),
            variadic: None,
            output: return_type,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(result_expr, None)],
        },
    }
}

fn generate_fabricator_impl(s: &ItemStruct) -> anyhow::Result<ItemImpl> {
    let s_ident = s.ident.clone();
    let struct_type = Type::Path(create_type_path(Path::from(s.ident.clone())));
    let impl_path: Path = parse_quote!(::mck::misc::Meta::<super::#s_ident>);
    let impl_trait = (None, impl_path, Default::default());
    let self_type = Type::Path(create_type_path(path!(Self)));
    let self_input = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: Default::default(),
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: Default::default(),
            elem: Box::new(self_type),
        })),
    });

    let first_fn = fabricate_first_fn(s, self_input.clone());
    let increment_fn = increment_fabricated_fn(s, self_input);

    Ok(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Generics::default(),
        trait_: Some(impl_trait),
        self_ty: Box::new(struct_type),
        brace_token: Default::default(),
        items: vec![ImplItem::Fn(first_fn), ImplItem::Fn(increment_fn)],
    })
}

fn generate_markable_impl(s: &ItemStruct) -> anyhow::Result<ItemImpl> {
    let mark_path = Path::from(s.ident.clone());
    let mark_type = Type::Path(create_type_path(mark_path.clone()));
    let mut abstr_path = mark_path;
    abstr_path.segments.insert(
        0,
        PathSegment {
            ident: create_ident("super"),
            arguments: PathArguments::None,
        },
    );

    let markable_item_type = ImplItemType {
        attrs: vec![],
        vis: Visibility::Inherited,
        defaultness: None,
        type_token: Default::default(),
        ident: create_ident("Mark"),
        generics: Default::default(),
        eq_token: Default::default(),
        ty: mark_type.clone(),
        semi_token: Default::default(),
    };

    let abstr_ref_type = Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: None,
        mutability: None,
        elem: Box::new(Type::Path(create_type_path(create_path_from_name("Self")))),
    });

    let abstr_self_arg = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: None,
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(abstr_ref_type),
    });

    let expr = Expr::Call(create_expr_call(
        Expr::Path(create_expr_path(path!(::std::default::Default::default))),
        Punctuated::new(),
    ));

    let create_clean_mark_fn = ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("create_clean_mark"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![abstr_self_arg]),
            variadic: None,
            output: ReturnType::Type(Default::default(), Box::new(mark_type)),
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(expr, None)],
        },
    };

    let struct_type = Type::Path(create_type_path(abstr_path));
    let impl_trait = (None, path!(::mck::refin::Markable), Default::default());
    Ok(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Generics::default(),
        trait_: Some(impl_trait),
        self_ty: Box::new(struct_type),
        brace_token: Default::default(),
        items: vec![
            ImplItem::Type(markable_item_type),
            ImplItem::Fn(create_clean_mark_fn),
        ],
    })
}

pub fn mark_path_rules() -> Vec<PathRule> {
    vec![PathRule {
        has_leading_colon: true,
        segments: vec![
            PathRuleSegment::Ident(String::from("mck")),
            PathRuleSegment::Convert(String::from("abstr"), String::from("refin")),
            PathRuleSegment::Wildcard,
        ],
    }]
}
