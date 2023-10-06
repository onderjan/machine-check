use anyhow::anyhow;
use proc_macro2::Span;
use quote::quote;
use syn::{
    punctuated::Punctuated, visit_mut::VisitMut, Ident, Member, Path, PathSegment, Stmt, Type,
};

pub struct ConversionScheme {
    prefix: String,
    scheme: String,
    self_name: String,
    convert_type_to_super: bool,
}

impl ConversionScheme {
    pub fn new(
        prefix: String,
        scheme: String,
        self_name: String,
        convert_type_to_super: bool,
    ) -> Self {
        ConversionScheme {
            prefix,
            scheme,
            self_name,
            convert_type_to_super,
        }
    }

    pub fn apply_to_stmt(&self, stmt: &mut Stmt) -> anyhow::Result<()> {
        let mut visitor = ConversionVisitor {
            scheme: self,
            result: Ok(()),
        };
        visitor.visit_stmt_mut(stmt);
        visitor.result
    }

    pub fn convert_type(&self, ty: &Type) -> anyhow::Result<Type> {
        if let Type::Reference(ty) = ty {
            let mut ty = ty.clone();
            *ty.elem = self.convert_type(&ty.elem)?;
            return Ok(Type::Reference(ty));
        }

        let Type::Path(ty) = ty else {
            return Err(anyhow!("Non-path type '{}' not supported", quote!(#ty)));
        };

        if ty.qself.is_some() {
            return Err(anyhow!(
                "Qualified-path type '{}' not supported",
                quote!(#ty)
            ));
        }

        let mut ty = ty.clone();
        ty.path = self.convert_type_path(&ty.path);

        Ok(Type::Path(ty))
    }

    fn convert_type_path(&self, path: &Path) -> Path {
        if !self.convert_type_to_super {
            return path.clone();
        }

        let mut path = path.clone();
        if path.leading_colon.is_some() {
            // do not convert
            return path;
        }

        let path_segments = &mut path.segments;
        // replace Self by type name
        for path_segment in path_segments.iter_mut() {
            if path_segment.ident == "Self" {
                path_segment.ident = Ident::new(self.self_name.as_str(), path_segment.ident.span());
            }
        }

        // TODO: select leading part of global path instead of hardcoded super
        path_segments.insert(
            0,
            PathSegment {
                ident: Ident::new("super", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
        );
        path
    }

    pub fn convert_name(&self, name: &str) -> String {
        let name = name.strip_prefix(&self.prefix).unwrap_or(name);
        format!("{}{}_{}", &self.prefix, &self.scheme, &name)
    }

    fn convert_ident(&self, ident: &Ident) -> Ident {
        Ident::new(
            self.convert_name(ident.to_string().as_str()).as_str(),
            ident.span(),
        )
    }

    fn convert_normal_path(&self, path: &Path) -> anyhow::Result<Path> {
        // only change idents
        if let Some(ident) = path.get_ident() {
            Ok(Path::from(self.convert_ident(ident)))
        } else {
            // the path must be global
            if path.leading_colon.is_none() {
                return Err(anyhow!(
                    "Non-ident local path '{}' not supported",
                    quote!(#path),
                ));
            }
            Ok(path.clone())
        }
    }
}

struct ConversionVisitor<'a> {
    scheme: &'a ConversionScheme,
    result: Result<(), anyhow::Error>,
}

impl<'a> VisitMut for ConversionVisitor<'a> {
    fn visit_pat_struct_mut(&mut self, node: &mut syn::PatStruct) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        if let Some(it) = &mut node.qself {
            self.visit_qself_mut(it);
        }
        // treat specially by considering struct path to be a type
        node.path = self.scheme.convert_type_path(&node.path);
        for mut el in Punctuated::pairs_mut(&mut node.fields) {
            let it = el.value_mut();
            self.visit_field_pat_mut(it);
        }
        if let Some(it) = &mut node.rest {
            self.visit_pat_rest_mut(it);
        }
    }

    fn visit_expr_struct_mut(&mut self, node: &mut syn::ExprStruct) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        if let Some(it) = &mut node.qself {
            self.visit_qself_mut(it);
        }
        // treat specially by considering struct path to be a type
        node.path = self.scheme.convert_type_path(&node.path);
        for mut el in Punctuated::pairs_mut(&mut node.fields) {
            let it = el.value_mut();
            self.visit_field_value_mut(it);
        }
        if let Some(it) = &mut node.rest {
            self.visit_expr_mut(it);
        }
    }

    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        self.visit_visibility_mut(&mut node.vis);
        self.visit_field_mutability_mut(&mut node.mutability);
        // treat specially by not going into field
        self.visit_type_mut(&mut node.ty);
    }

    fn visit_member_mut(&mut self, _: &mut Member) {
        // do not go into the member
    }

    fn visit_type_mut(&mut self, i: &mut Type) {
        match self.scheme.convert_type(i) {
            Ok(ok) => *i = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
        // do not propagate
    }
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        *i = self.scheme.convert_ident(i);
        // do not propagate
    }
    fn visit_path_mut(&mut self, i: &mut Path) {
        match self.scheme.convert_normal_path(i) {
            Ok(ok) => *i = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
        // do not propagate
    }
}
