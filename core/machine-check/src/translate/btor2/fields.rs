use btor2rs::{Nid, Node, Sid, Source, SourceType};
use proc_macro2::Span;
use syn::{Field, Ident, Type};

use super::{
    util::{create_nid_ident, create_sid_type, create_single_bit_type},
    Translator,
};

impl Translator {
    pub(super) fn create_input_fields(&self) -> Result<Vec<Field>, anyhow::Error> {
        // add inputs and states without init or next to input fields
        let mut fields = Vec::new();
        for (nid, node) in &self.btor2.nodes {
            if let Node::Source(Source {
                ty: SourceType::Input,
                sid,
            }) = node
            {
                fields.push(self.create_nid_field(*nid, *sid)?);
            }
        }

        for (nid, state_info) in &self.state_info_map {
            // if state has no init or no next, it can be treated as input
            if state_info.init.is_none() || state_info.next.is_none() {
                fields.push(self.create_nid_field(*nid, state_info.sid)?);
            }
        }
        Ok(fields)
    }

    pub(super) fn create_state_fields(&self) -> Result<Vec<Field>, anyhow::Error> {
        let mut fields = Vec::new();
        for (nid, state_info) in &self.state_info_map {
            // if state has next, it is a field
            if state_info.next.is_some() {
                fields.push(self.create_nid_field(*nid, state_info.sid)?);
            }
        }
        self.add_drain_fields(&mut fields);
        Ok(fields)
    }

    fn add_drain_fields(&self, state_fields: &mut Vec<Field>) {
        let bit_type = create_single_bit_type();
        // add 'constrained' state field
        let constrained_ident = Ident::new("constrained", Span::call_site());
        state_fields.push(create_field(constrained_ident, bit_type.clone()));
        // add 'safe' state field
        let safe_ident = Ident::new("safe", Span::call_site());
        state_fields.push(create_field(safe_ident, bit_type));
    }

    fn create_nid_field(&self, nid: Nid, sid: Sid) -> Result<Field, anyhow::Error> {
        let ident = create_nid_ident(nid);
        let ty = create_sid_type(&self.btor2, sid)?;
        Ok(Field {
            attrs: vec![],
            vis: syn::Visibility::Public(Default::default()),
            mutability: syn::FieldMutability::None,
            ident: Some(ident),
            colon_token: Some(Default::default()),
            ty,
        })
    }
}

fn create_field(ident: Ident, ty: Type) -> Field {
    Field {
        attrs: vec![],
        vis: syn::Visibility::Public(Default::default()),
        mutability: syn::FieldMutability::None,
        ident: Some(ident),
        colon_token: Some(Default::default()),
        ty,
    }
}
