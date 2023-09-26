use syn::ItemStruct;

use super::mark_type_path::TypePathVisitor;

pub fn transcribe_struct(s: &ItemStruct) -> anyhow::Result<ItemStruct> {
    let mut mark_s = s.clone();
    TypePathVisitor::new().visit_struct(&mut mark_s);
    Ok(mark_s)
}
