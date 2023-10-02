use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::Brace, BinOp, Block, Expr, ExprBinary, ExprReference,
    ExprStruct, FnArg, Generics, Ident, ImplItem, ImplItemFn, ImplItemType, Item, ItemImpl,
    ItemMod, ItemStruct, Pat, PatType, Path, PathArguments, PathSegment, Receiver, ReturnType,
    Signature, Stmt, Type, TypeReference, Visibility,
};
use syn_path::path;

use crate::transcription::util::{
    create_expr_call, create_expr_field, create_expr_path, create_field_value, create_ident,
    create_pat_ident, create_path_from_ident, create_path_from_name, create_type_path,
    path_rule::{self, PathRule, PathRuleSegment},
};

use self::{mark_fn::transcribe_item_impl, mark_stmt::create_join_stmt};

mod mark_fn;
mod mark_stmt;

pub fn apply(file: &mut syn::File) -> anyhow::Result<()> {
    // the mark will be in a new module under the abstract

    // create items to add to the module
    let mut mark_file_items = Vec::<Item>::new();
    for item in &file.items {
        match item {
            Item::Struct(s) => {
                apply_transcribed_item_struct(&mut mark_file_items, s)?;
            }
            Item::Impl(i) => {
                mark_file_items.push(Item::Impl(transcribe_item_impl(i)?));
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
        ident: Ident::new("mark", Span::call_site()),
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
    let join_impl = generate_join_impl(&s)?;
    let possibility_impl = generate_possibility_impl(&s)?;
    let markable_impl = generate_markable_impl(&s)?;
    // add struct
    items.push(Item::Struct(s));
    // add implementations of join, possibility, and markable
    items.push(Item::Impl(join_impl));
    items.push(Item::Impl(possibility_impl));
    items.push(Item::Impl(markable_impl));

    Ok(())
}

fn generate_join_impl(s: &ItemStruct) -> anyhow::Result<ItemImpl> {
    let struct_type = Type::Path(create_type_path(Path::from(s.ident.clone())));
    let join_impl_trait = (None, path!(::mck::mark::Join), Default::default());
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

fn generate_first_possibility_fn(s: &ItemStruct, self_input: FnArg) -> ImplItemFn {
    let possibility_path = path!(Self::Possibility);
    let return_type = ReturnType::Type(
        Default::default(),
        Box::new(Type::Path(create_type_path(possibility_path.clone()))),
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
            Expr::Path(create_expr_path(path!(
                ::mck::Possibility::first_possibility
            ))),
            Punctuated::from_iter(vec![self_ref_expr]),
        ));

        let field_value = create_field_value(index, field, init_expr);

        struct_expr_fields.push(field_value);
    }

    let struct_expr = Expr::Struct(ExprStruct {
        attrs: vec![],
        qself: None,
        path: possibility_path,
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
            ident: create_ident("first_possibility"),
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

fn generate_increment_possibility_fn(s: &ItemStruct, self_input: FnArg) -> ImplItemFn {
    let possibility_ident = create_ident("possibility");
    let possibility_type = Type::Path(create_type_path(path!(Self::Possibility)));
    let possibility_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(possibility_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: None,
            mutability: Some(Default::default()),
            elem: Box::new(possibility_type),
        })),
    });

    let return_type = ReturnType::Type(
        Default::default(),
        Box::new(Type::Path(create_type_path(path!(bool)))),
    );

    let mut result_expr = None;

    /*
            fn increment_possibility(&self, possibility: &mut super::Input) -> bool {
                ::mck::Possibility::increment_possibility(&self.input_2, &mut possibility.input_2)
                    || self.input_3.increment_possibility(&mut possibility.input_3)
            }
    */

    for (index, field) in s.fields.iter().enumerate() {
        let self_expr_path = create_expr_path(path!(self));
        let possibility_expr_path =
            create_expr_path(create_path_from_ident(possibility_ident.clone()));

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
        let possibility_expr = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: Some(Default::default()),
            expr: Box::new(Expr::Field(create_expr_field(
                Expr::Path(possibility_expr_path),
                index,
                field,
            ))),
        });
        let func_expr = Expr::Path(create_expr_path(path!(
            ::mck::Possibility::increment_possibility
        )));
        let expr = Expr::Call(create_expr_call(
            func_expr,
            Punctuated::from_iter(vec![self_expr, possibility_expr]),
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
            ident: create_ident("increment_possibility"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input, possibility_input]),
            variadic: None,
            output: return_type,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(result_expr, None)],
        },
    }
}

fn generate_possibility_impl_item_type(s: &ItemStruct) -> ImplItemType {
    let mut path = create_path_from_ident(s.ident.clone());
    path.segments.insert(
        0,
        PathSegment {
            ident: create_ident("super"),
            arguments: PathArguments::None,
        },
    );
    ImplItemType {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: Default::default(),
        type_token: Default::default(),
        ident: create_ident("Possibility"),
        generics: Default::default(),
        eq_token: Default::default(),
        ty: Type::Path(create_type_path(path)),
        semi_token: Default::default(),
    }
}

fn generate_possibility_impl(s: &ItemStruct) -> anyhow::Result<ItemImpl> {
    let struct_type = Type::Path(create_type_path(Path::from(s.ident.clone())));
    let impl_trait = (None, path!(::mck::Possibility), Default::default());
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

    let possibility_item_type = generate_possibility_impl_item_type(s);

    let first_possibility_fn = generate_first_possibility_fn(s, self_input.clone());
    let next_possibility_fn = generate_increment_possibility_fn(s, self_input);

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
            ImplItem::Type(possibility_item_type),
            ImplItem::Fn(first_possibility_fn),
            ImplItem::Fn(next_possibility_fn),
        ],
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
    let impl_trait = (None, path!(::mck::mark::Markable), Default::default());
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

    /*impl ::mck::mark::Markable for super::State {
        type Mark = State;

        fn create_clean_mark(&self) -> Self::Mark {
            Default::default()
        }
    }*/
}

pub fn mark_path_rules() -> Vec<PathRule> {
    vec![PathRule {
        has_leading_colon: true,
        segments: vec![
            PathRuleSegment::Ident(String::from("mck")),
            PathRuleSegment::Convert(
                String::from("ThreeValuedBitvector"),
                String::from("MarkBitvector"),
            ),
        ],
    }]
}
