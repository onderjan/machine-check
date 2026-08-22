/*
use std::collections::HashSet;

use crate::wir::{WBlock, WIdent, WStmt, ZSsa};

pub fn used_open_idents(block: &WBlock<ZSsa>) -> HashSet<WIdent> {
    let mut used_open = UsedOpen { scopes: vec![] };

    used_open.open_block_idents(block)
}

struct UsedOpen {
    scopes: Vec<HashSet<WIdent>>,
}

impl UsedOpen {
    fn open_block_idents(&mut self, block: &WBlock<ZSsa>) -> HashSet<WIdent> {
        let mut result = HashSet::new();

        self.scopes.push(HashSet::new());

        for stmt in &block.stmts {
            result.extend(self.open_stmt_idents(stmt));
        }

        self.scopes.pop();
        result
    }

    fn open_stmt_idents(&mut self, stmt: &WStmt<ZSsa>) -> HashSet<WIdent> {
        match stmt {
            WStmt::Assign(stmt_assign) => {
                let right_idents = stmt_assign.right.idents();

                let mut result = HashSet::new();

                for right_ident in right_idents {
                    if self.is_open(&right_ident) {
                        result.insert(right_ident);
                    }
                }

                self.scopes
                    .last_mut()
                    .expect("Last scope should be present")
                    .insert(stmt_assign.left.clone());

                result
            }
            WStmt::If(stmt_if) => {
                let mut result = HashSet::new();
                if self.is_open(&stmt_if.condition.ident) {
                    result.insert(stmt_if.condition.ident.clone());
                }

                result.extend(self.open_block_idents(&stmt_if.then_block));
                result.extend(self.open_block_idents(&stmt_if.else_block));

                result
            }
        }
    }

    fn is_open(&self, ident: &WIdent) -> bool {
        for scope in &self.scopes {
            if scope.contains(ident) {
                return false;
            }
        }
        true
    }
}
*/
