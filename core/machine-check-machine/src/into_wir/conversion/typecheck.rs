use std::rc::Rc;

use indexmap::IndexMap;

use crate::wir::{WBasicType, WDescription, WIdent, WItemStruct, WSignature, YSsa};

pub fn typecheck(description: WDescription<YSsa>) {
    let mut types = IndexMap::new();

    for item_struct in description.structs {
        let key = RPath {
            name: item_struct.ident.clone(),
        };
        let value = Type::Struct(Struct { inner: item_struct });
        types.insert(key, Rc::new(value));
    }

    let checker = Typechecker { types };

    eprintln!("Typechecker: {:#?}", checker);

    let mut signatures = IndexMap::new();

    for item_impl in description.impls {
        for item_fn in item_impl.impl_item_fns {
            let key = RPath {
                name: item_fn.signature.ident.clone(),
            };

            /*let mut inputs = Vec::new();
            for input in &item_fn.signature.inputs {
                let path = RPath {
                    name: input.ty.clone(),
                };
                inputs.push((input.ident.clone(), checker.get_type(&path)));
            }*/

            let value = Signature::ImplFn(ImplFn {
                signature: item_fn.signature,
            });
            signatures.insert(key, value);
        }
    }
    eprintln!("Signatures: {:#?}", signatures);
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct RPath {
    name: WIdent,
}

#[derive(Debug, Hash)]
struct Struct {
    inner: WItemStruct<WBasicType>,
}

#[derive(Debug, Hash)]
enum Type {
    Struct(Struct),
    Unknown,
}

#[derive(Debug, Hash)]
struct ImplFn {
    signature: WSignature<YSsa>,
}

#[derive(Debug, Hash)]
enum Signature {
    ImplFn(ImplFn),
}

#[derive(Debug)]
struct Typechecker {
    types: IndexMap<RPath, Rc<Type>>,
}

impl Typechecker {
    fn get_type(&self, ident: RPath) -> Rc<Type> {
        if let Some(ty) = self.types.get(&ident) {
            Rc::clone(&ty)
        } else {
            // TODO error
            Rc::new(Type::Unknown)
        }
    }
}
