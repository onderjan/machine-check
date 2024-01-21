use std::collections::{HashMap, HashSet};

use syn::{
    visit_mut::{self, VisitMut},
    Block, ExprBlock, Ident, ItemStruct, Stmt,
};
use syn_path::path;

use crate::{
    support::{field_manipulate, local::construct_prefixed_ident, local_types::find_local_types},
    util::{
        create_assign, create_expr_call, create_expr_ident, create_expr_path, create_let_bare,
        create_path_from_ident, extract_else_block_with_token, extract_expr_ident, ArgType,
    },
    MachineDescription,
};

use super::util::generate_derive_attribute;

use quote::quote;

use super::{
    support::path_rules::{PathRule, PathRuleSegment, PathRules},
    MachineError,
};

use syn::{Expr, ExprIf};

pub(crate) fn create_abstract_machine(
    ssa_machine: &MachineDescription,
) -> Result<MachineDescription, MachineError> {
    // expecting the concrete machine in SSA form
    let mut abstract_machine = ssa_machine.clone();
    // apply transcription to types using path rule transcriptor
    path_rules().apply_to_items(&mut abstract_machine.items)?;

    // add default derive attributes to the structs
    // that easily allow us to make unknown inputs/states
    for item in abstract_machine.items.iter_mut() {
        Visitor {
            tmps: HashMap::new(),
            tmp_counter: 0,
        }
        .visit_item_mut(item);
    }

    // add field-manipulate to items
    field_manipulate::apply_to_items(&mut abstract_machine.items, "abstr")?;

    Ok(abstract_machine)
}

struct Visitor {
    tmps: HashMap<Ident, Ident>,
    tmp_counter: usize,
}
impl VisitMut for Visitor {
    fn visit_item_struct_mut(&mut self, s: &mut ItemStruct) {
        s.attrs
            .push(generate_derive_attribute(quote!(::std::default::Default)));
    }

    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        // visit first
        visit_mut::visit_impl_item_fn_mut(self, impl_item_fn);

        // perform transitive closure on temporaries
        let mut local_tmps_closure = HashMap::new();

        for (tmp_ident, mut closure_ident) in self.tmps.iter() {
            while let Some(get_ident) = self.tmps.get(closure_ident) {
                closure_ident = get_ident;
            }
            local_tmps_closure.insert(tmp_ident.clone(), closure_ident.clone());
        }

        let local_types = find_local_types(impl_item_fn);

        // add bare let for every temporary
        let mut local_stmts = Vec::new();
        for tmp in local_tmps_closure {
            let ty = local_types
                .get(&tmp.1)
                .expect("Original for temporary should be typed");
            local_stmts.push(create_let_bare(tmp.0, Some(ty.clone())));
        }
        local_stmts.append(&mut impl_item_fn.block.stmts);
        impl_item_fn.block.stmts = local_stmts;
        // clear temporaries
        self.tmps.clear();
    }

    fn visit_expr_if_mut(&mut self, expr_if: &mut ExprIf) {
        // TODO: integrate abstract conditions better
        self.convert_expr_if(expr_if);
        // propagate afterwards
        visit_mut::visit_expr_if_mut(self, expr_if);
    }
}

impl Visitor {
    fn join_statements(
        &mut self,
        mut then_stmts: Vec<Stmt>,
        mut else_stmts: Vec<Stmt>,
        condition: &Ident,
        if_counter: usize,
    ) -> (Vec<Stmt>, HashMap<Ident, Ident>) {
        // convert every assignment to assignment to a new temporary
        let mut stmts = Vec::new();
        let mut then_set = HashSet::new();
        let mut else_set = HashSet::new();
        find_temporaries(&then_stmts, &mut then_set);
        find_temporaries(&else_stmts, &mut else_set);
        let mut then_temporary_map = HashMap::new();
        let mut else_temporary_map = HashMap::new();
        let mut phi_stmts = Vec::new();
        for left_ident in then_set {
            if else_set.contains(&left_ident) {
                // is in both, convert to temporary and create phi statement
                let then_tmp_ident =
                    construct_prefixed_ident(&format!("then_{}", if_counter), &left_ident);
                let else_tmp_ident =
                    construct_prefixed_ident(&format!("else_{}", if_counter), &left_ident);
                then_temporary_map.insert(left_ident.clone(), then_tmp_ident.clone());
                else_temporary_map.insert(left_ident.clone(), else_tmp_ident.clone());
                phi_stmts.push(create_assign(
                    left_ident,
                    create_expr_call(
                        create_expr_path(path!(::mck::abstr::Phi::phi)),
                        vec![
                            (ArgType::Normal, create_expr_ident(then_tmp_ident)),
                            (ArgType::Normal, create_expr_ident(else_tmp_ident)),
                            (ArgType::Normal, create_expr_ident(condition.clone())),
                        ],
                    ),
                    true,
                ));
            }
        }
        convert_to_temporaries(&mut then_stmts, &then_temporary_map);
        convert_to_temporaries(&mut else_stmts, &else_temporary_map);

        stmts.extend(then_stmts);
        stmts.extend(else_stmts);
        stmts.extend(phi_stmts);

        let mut tmps = HashMap::new();
        tmps.extend(
            then_temporary_map
                .into_iter()
                .map(|(orig_ident, tmp_ident)| (tmp_ident, orig_ident)),
        );
        tmps.extend(
            else_temporary_map
                .into_iter()
                .map(|(orig_ident, tmp_ident)| (tmp_ident, orig_ident)),
        );

        (stmts, tmps)
    }

    fn convert_expr_if(&mut self, expr_if: &mut ExprIf) {
        let Expr::Call(cond_expr_call) = expr_if.cond.as_mut() else {
            return;
        };
        let Expr::Path(cond_expr_path) = cond_expr_call.func.as_mut() else {
            return;
        };
        if cond_expr_path.path != path!(::mck::abstr::Test::is_true) {
            return;
        }
        if cond_expr_call.args.len() != 1 {
            // TODO: replace with result
            panic!("Invalid number of arguments for Test");
        }
        let condition =
            extract_expr_ident(&cond_expr_call.args[0]).expect("Condition should be ident");

        // split into three possibilities:
        // 1. must be true (perform only then)
        // 2. must be false (perform only else)
        // 3. otherwise (perform both and join them)
        // that is:
        // if must_be_true(cond) { then_block }
        // else { if must_be_false(cond) { else_block } else { join_block } }

        // leave then block as-is, just replace the condition path
        cond_expr_path.path = path!(::mck::abstr::Test::must_be_true);
        let then_block = expr_if.then_branch.clone();

        // create a new condition in the else block
        let (else_block, else_token) = extract_else_block_with_token(&expr_if.else_branch)
            .expect("Expected if with else block");
        let mut must_be_false_call = cond_expr_call.clone();
        let Expr::Path(must_be_false_path) = must_be_false_call.func.as_mut() else {
            panic!("Should be path");
        };

        must_be_false_path.path = path!(::mck::abstr::Test::must_be_false);
        let (both_stmts, both_tmps) = self.join_statements(
            then_block.stmts,
            else_block.stmts.clone(),
            condition,
            self.tmp_counter,
        );
        self.tmp_counter += 1;
        self.tmps.extend(both_tmps);
        // TODO
        let both_block = ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token: Default::default(),
                stmts: both_stmts,
            },
        };
        let new_if = Expr::If(ExprIf {
            attrs: vec![],
            if_token: expr_if.if_token,
            cond: Box::new(Expr::Call(must_be_false_call)),
            then_branch: else_block.clone(),
            else_branch: Some((*else_token, Box::new(Expr::Block(both_block)))),
        });

        expr_if.else_branch = Some((
            *else_token,
            Box::new(Expr::Block(ExprBlock {
                attrs: vec![],
                label: None,
                block: Block {
                    brace_token: Default::default(),
                    stmts: vec![Stmt::Expr(new_if, Some(Default::default()))],
                },
            })),
        ));
    }
}

fn find_temporaries(stmts: &[Stmt], temporary_set: &mut HashSet<Ident>) {
    for stmt in stmts {
        match stmt {
            Stmt::Expr(expr, Some(_)) => match expr {
                Expr::Assign(assign) => {
                    // insert to temporary map
                    let left_ident = extract_expr_ident(&assign.left)
                        .expect("Left side of assignment should be ident");
                    temporary_set.insert(left_ident.clone());
                }
                Expr::Block(expr_block) => {
                    find_temporaries(&expr_block.block.stmts, temporary_set);
                }
                Expr::If(expr_if) => {
                    find_temporaries(&expr_if.then_branch.stmts, temporary_set);
                    let Some((_else_token, else_block)) = &expr_if.else_branch else {
                        // TODO: replace with result
                        panic!("If without else");
                    };
                    let Expr::Block(else_expr_block) = else_block.as_ref() else {
                        // TODO: replace with result
                        panic!("Non-block else");
                    };
                    find_temporaries(&else_expr_block.block.stmts, temporary_set);
                }
                _ => panic!("Unexpected expression type: {:?}", stmt),
            },
            _ => panic!("Unexpected statement type: {:?}", stmt),
        }
    }
}

fn convert_to_temporaries(stmts: &mut [Stmt], temporary_map: &HashMap<Ident, Ident>) {
    for stmt in stmts.iter_mut() {
        match stmt {
            Stmt::Expr(expr, Some(semi)) => match expr {
                Expr::Assign(assign) => {
                    let Expr::Path(left_path) = assign.left.as_ref() else {
                        panic!("Unexpected non-path left");
                    };

                    if !(left_path.path.leading_colon.is_none()
                        && left_path.path.segments.len() == 1
                        && left_path.path.segments[0].arguments.is_none())
                    {
                        panic!("Unexpected non-ident left");
                    }

                    // try to find the temporary and convert if we have it
                    let left_ident = &left_path.path.segments[0].ident;
                    if let Some(tmp_ident) = temporary_map.get(left_ident) {
                        let mut assign = assign.clone();
                        assign.left =
                            Box::new(create_expr_path(create_path_from_ident(tmp_ident.clone())));
                        *stmt = Stmt::Expr(Expr::Assign(assign), Some(*semi));
                    }
                }
                Expr::Block(expr_block) => {
                    convert_to_temporaries(&mut expr_block.block.stmts, temporary_map);
                }
                Expr::If(expr_if) => {
                    convert_to_temporaries(&mut expr_if.then_branch.stmts, temporary_map);
                    let Some((_else_token, else_block)) = &mut expr_if.else_branch else {
                        // TODO: replace with result
                        panic!("If without else");
                    };
                    let Expr::Block(else_expr_block) = else_block.as_mut() else {
                        // TODO: replace with result
                        panic!("Non-block else");
                    };
                    convert_to_temporaries(&mut else_expr_block.block.stmts, temporary_map);
                }
                _ => panic!("Unexpected expression type: {:?}", stmt),
            },
            _ => panic!("Unexpected statement type: {:?}", stmt),
        }
    }
}

fn path_rules() -> PathRules {
    PathRules::new(vec![
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Convert(String::from("concr"), String::from("abstr")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Match(String::from("forward")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Match(String::from("attr")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::Wildcard],
        },
    ])
}
