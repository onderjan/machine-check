use anyhow::anyhow;
use syn::{punctuated::Punctuated, visit_mut::VisitMut, Expr, Ident, Member, Path, Stmt, Type};

use super::{create_path_from_ident, create_type_path, path_rules::PathRules};

#[derive(Clone)]
pub struct StructRules {
    self_ty_ident: Ident,
    normal_rules: PathRules,
    type_rules: PathRules,
}

impl StructRules {
    pub fn new(self_ty_ident: Ident, normal_rules: PathRules, type_rules: PathRules) -> Self {
        StructRules {
            self_ty_ident,
            normal_rules,
            type_rules,
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

    pub fn apply_to_expr(&self, expr: &mut Expr) -> anyhow::Result<()> {
        let mut visitor = ConversionVisitor {
            scheme: self,
            result: Ok(()),
        };
        visitor.visit_expr_mut(expr);
        visitor.result
    }

    pub fn convert_type(&self, ty: Type) -> anyhow::Result<Type> {
        if let Type::Reference(ty) = ty {
            let mut ty = ty;
            *ty.elem = self.convert_type(*ty.elem)?;
            return Ok(Type::Reference(ty));
        }

        let Type::Path(ty) = ty else {
            return Err(anyhow!("Non-path type not supported"));
        };

        if ty.qself.is_some() {
            return Err(anyhow!("Qualified-path type not supported"));
        }

        Ok(create_type_path(self.convert_type_path(&ty.path)?))
    }

    fn convert_type_path(&self, path: &Path) -> anyhow::Result<Path> {
        if path.leading_colon.is_some() {
            // just apply the rules
            return self.type_rules.convert_path(path.clone());
        }

        let mut path_segments = path.segments.clone();
        // replace Self by type name
        for path_segment in path_segments.iter_mut() {
            if path_segment.ident == "Self" {
                path_segment.ident = self.self_ty_ident.clone();
            }
        }

        // apply the rules
        self.type_rules.convert_path(path.clone())
    }

    pub fn convert_normal_ident(&self, ident: Ident) -> anyhow::Result<Ident> {
        let path = self.convert_normal_path(create_path_from_ident(ident))?;
        Ok(path
            .get_ident()
            .expect("Identifier should be converted to identifier")
            .clone())
    }

    pub fn convert_normal_path(&self, path: Path) -> anyhow::Result<Path> {
        self.normal_rules.convert_path(path)
    }
}

struct ConversionVisitor<'a> {
    scheme: &'a StructRules,
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
        match self.scheme.convert_type_path(&node.path) {
            Ok(ok) => node.path = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
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
        match self.scheme.convert_type_path(&node.path) {
            Ok(ok) => node.path = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
        for mut el in node.fields.pairs_mut() {
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

    fn visit_type_mut(&mut self, ty: &mut Type) {
        let result = self.scheme.convert_type(ty.clone());
        match result {
            Ok(ok) => *ty = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
        // do not propagate
    }

    fn visit_member_mut(&mut self, _: &mut Member) {
        // do not go into the member
    }

    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        let result = self.scheme.convert_normal_ident(ident.clone());
        match result {
            Ok(ok) => *ident = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
        // do not propagate
    }
    fn visit_path_mut(&mut self, path: &mut Path) {
        let result = self.scheme.convert_normal_path(path.clone());
        match result {
            Ok(ok) => *path = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
        // do not propagate
    }
}
