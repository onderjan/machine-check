use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::Brace, Block, Expr, ExprField, FnArg, Generics, Ident, ImplItem,
    ImplItemFn, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, Member, Pat, PatIdent, PatType, Path,
    PathSegment, Receiver, ReturnType, Signature, Type, TypePath, TypeReference,
};
use syn_path::path;

use crate::transcription::util::{
    create_expr_field, create_expr_path, create_ident, create_pat_ident, create_path_from_ident,
    create_path_from_name, create_type_path,
    path_rule::{self, PathRule, PathRuleSegment},
};

use anyhow::anyhow;

use quote::quote;

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
    // add struct
    items.push(Item::Struct(s));
    // add implementation of join
    items.push(Item::Impl(join_impl));

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

pub fn mark_path_rules() -> Vec<PathRule> {
    vec![
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Ident(String::from("mck")),
                PathRuleSegment::Convert(
                    String::from("ThreeValuedArray"),
                    String::from("MarkArray"),
                ),
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Ident(String::from("mck")),
                PathRuleSegment::Convert(
                    String::from("ThreeValuedBitvector"),
                    String::from("MarkBitvector"),
                ),
            ],
        },
    ]
}
