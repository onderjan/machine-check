use crate::wir::{WIdent, WPartialPath, WPartialSegment};

/*
fn lower_item_struct(
    _ctx: &mut WInferredContext,
    item_struct: WItemStruct,
) -> Result<WItemStruct, Errors> {
    let derives = item_struct
        .derives
        .into_iter()
        .map(lower_basic_path)
        .collect();
    let fields = item_struct.fields;
    Ok(WItemStruct {
        visibility: item_struct.visibility,
        derives,
        ident: item_struct.ident,
        fields,
    })
}

fn lower_item_impl(
    ctx: &mut WInferredContext,
    item_impl: WItemImpl<YTac>,
) -> Result<WItemImpl<YLowered>, Errors> {
    let mut impl_item_fns = Vec::new();

    for impl_item_fn in item_impl.impl_item_fns {
        impl_item_fns.push(lower_item_fn(
            ctx,
            /*Some(&item_impl.self_ty),*/ impl_item_fn,
        ));
    }

    let impl_item_types = item_impl
        .impl_item_types
        .into_iter()
        .map(|type_item| WImplItemType {
            visibility: type_item.visibility,
            left_ident: type_item.left_ident,
            right_path: lower_basic_path(type_item.right_path),
        })
        .collect();

    let impl_item_fns = Errors::flat_result(impl_item_fns);

    match impl_item_fns {
        Ok(impl_item_fns) => Ok(WItemImpl {
            self_ty: item_impl.self_ty,
            trait_: item_impl.trait_,
            impl_item_types,
            impl_item_fns,
        }),
        Err(err) => Err(err),
    }
}
    */

pub fn lower_basic_path(path: WPartialPath) -> WPartialPath {
    if path.starts_with_absolute(&["machine_check", "Bitvector"])
        || path.starts_with_absolute(&["machine_check", "Unsigned"])
        || path.starts_with_absolute(&["machine_check", "Signed"])
    {
        let mut path = path_start_to_mck_concr(path);
        path.segments[2].ident.set_name(String::from("Bitvector"));
        return path;
    }

    if path.starts_with_absolute(&["machine_check", "BitvectorArray"]) {
        let mut path = path_start_to_mck_concr(path);
        path.segments[2].ident.set_name(String::from("Array"));
        return path;
    }

    if path.starts_with_absolute(&["machine_check", "Input"])
        || path.starts_with_absolute(&["machine_check", "State"])
        || path.starts_with_absolute(&["machine_check", "Param"])
        || path.starts_with_absolute(&["machine_check", "Machine"])
    {
        return path_start_to_mck_concr(path);
    }

    if path.starts_with_absolute(&["machine_check", "internal"]) {
        let mut path = path;
        path.segments[0].ident.set_name(String::from("mck"));
        path.segments[1].ident.set_name(String::from("concr"));
        return path;
    }
    path
}

fn path_start_to_mck_concr(path: WPartialPath) -> WPartialPath {
    path_start_to_mck_str("concr", path)
}

fn path_start_to_mck_str(str: &str, mut path: WPartialPath) -> WPartialPath {
    let first_ident = &mut path.segments[0].ident;
    first_ident.set_name(String::from("mck"));
    let first_ident_span = first_ident.span();
    path.segments.insert(
        1,
        WPartialSegment {
            ident: WIdent::new(String::from(str), first_ident_span),
            generics: None,
        },
    );
    path
}
