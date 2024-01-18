use std::collections::HashMap;

use syn::{
    visit_mut::{self, VisitMut},
    Block, ExprBlock, Ident, ItemStruct, Stmt,
};
use syn_path::path;

use crate::{
    util::{
        create_assign, create_expr_call, create_expr_ident, create_expr_path, create_let_bare,
        create_path_from_ident, ArgType,
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

pub(crate) fn apply(machine: &mut MachineDescription) -> Result<(), MachineError> {
    // apply transcription to types using path rule transcriptor
    path_rules().apply_to_items(&mut machine.items)?;

    // add default derive attributes to the structs
    // that easily allow us to make unknown inputs/states
    struct Visitor {
        tmps: Vec<Ident>,
    }
    impl VisitMut for Visitor {
        fn visit_item_struct_mut(&mut self, s: &mut ItemStruct) {
            s.attrs
                .push(generate_derive_attribute(quote!(::std::default::Default)));
        }

        fn visit_impl_item_fn_mut(&mut self, item_fn: &mut syn::ImplItemFn) {
            // visit first
            visit_mut::visit_impl_item_fn_mut(self, item_fn);
            // add local temporaries
            let mut local_tmps = Vec::new();
            local_tmps.append(&mut self.tmps);
            let mut local_stmts = Vec::new();
            for tmp in local_tmps {
                local_stmts.push(create_let_bare(tmp));
            }
            local_stmts.append(&mut item_fn.block.stmts);
            item_fn.block.stmts = local_stmts;
        }

        fn visit_expr_if_mut(&mut self, expr_if: &mut ExprIf) {
            // TODO: integrate abstract conditions better
            if let Expr::Call(cond_expr_call) = expr_if.cond.as_mut() {
                if let Expr::Path(cond_expr_path) = cond_expr_call.func.as_mut() {
                    if cond_expr_path.path.leading_colon.is_some() {
                        let segments = &mut cond_expr_path.path.segments;

                        // TODO: integrate the special conditions better
                        if segments.len() == 4
                            && &segments[0].ident.to_string() == "mck"
                            && &segments[1].ident.to_string() == "abstr"
                            && &segments[2].ident.to_string() == "Test"
                            && &segments[3].ident.to_string() == "is_true"
                        {
                            if cond_expr_call.args.len() != 1 {
                                // TODO: replace with result
                                panic!("Invalid number of arguments for Test");
                            }
                            let Expr::Path(ref condition_path) = cond_expr_call.args[0] else {
                                panic!("Unexpected non-path condition");
                            };
                            if !(condition_path.path.leading_colon.is_none()
                                && condition_path.path.segments.len() == 1
                                && condition_path.path.segments[0].arguments.is_none())
                            {
                                panic!("Unexpected non-ident condition");
                            }
                            // create a temporary
                            let condition = &condition_path.path.segments[0].ident;

                            // split into three possibilities:
                            // 1. must be true (perform only then)
                            // 2. must be false (perform only else)
                            // 3. otherwise (perform both and join them)
                            // that is:
                            // if must_be_true(cond) { then_block }
                            // else { if must_be_false(cond) { else_block } else { join_block } }

                            // leave then block as-is, just replace the condition
                            segments[3].ident =
                                Ident::new("must_be_true", segments[3].ident.span());
                            let then_block = expr_if.then_branch.clone();

                            // create a new condition in the else block
                            let Some((else_token, else_block)) =
                                std::mem::take(&mut expr_if.else_branch)
                            else {
                                // TODO: replace with result
                                panic!("If without else");
                            };
                            let Expr::Block(else_expr_block) = *else_block else {
                                // TODO: replace with result
                                panic!("Non-block else");
                            };
                            let else_block = else_expr_block.block;
                            let mut must_be_false_call = cond_expr_call.clone();
                            let Expr::Path(must_be_false_path) = must_be_false_call.func.as_mut()
                            else {
                                panic!("Should be path");
                            };

                            must_be_false_path.path.segments[3].ident = Ident::new(
                                "must_be_false",
                                must_be_false_path.path.segments[3].ident.span(),
                            );
                            let (both_stmts, both_tmps) =
                                join_statements(&then_block.stmts, &else_block.stmts, condition);
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
                                then_branch: else_block,
                                else_branch: Some((else_token, Box::new(Expr::Block(both_block)))),
                            });

                            expr_if.else_branch = Some((
                                else_token,
                                Box::new(Expr::Block(ExprBlock {
                                    attrs: vec![],
                                    label: None,
                                    block: Block {
                                        brace_token: Default::default(),
                                        stmts: vec![Stmt::Expr(new_if, Some(Default::default()))],
                                    },
                                })),
                            ));

                            //todo!();
                        }
                    }
                }
            }
        }
    }
    for item in machine.items.iter_mut() {
        Visitor { tmps: vec![] }.visit_item_mut(item);
    }
    Ok(())
}

fn join_statements(
    then_stmts: &[Stmt],
    else_stmts: &[Stmt],
    condition: &Ident,
) -> (Vec<Stmt>, Vec<Ident>) {
    // convert every assignment to assignment to a new temporary
    let mut stmts = Vec::new();
    let (then_stmts, then_temporary_map) = convert_to_temporaries(then_stmts, "then");
    let (else_stmts, mut else_temporary_map) = convert_to_temporaries(else_stmts, "else");
    stmts.extend(then_stmts);
    stmts.extend(else_stmts);
    // join the then/else temporaries
    let mut tmps = Vec::new();
    for (left_ident, then_tmp_ident) in then_temporary_map {
        let else_tmp_ident = else_temporary_map.remove(&left_ident).unwrap();
        tmps.push(then_tmp_ident.clone());
        tmps.push(else_tmp_ident.clone());
        stmts.push(create_assign(
            left_ident,
            create_expr_call(
                create_expr_path(path!(::mck::abstr::Join::join)),
                vec![
                    (ArgType::Normal, create_expr_ident(then_tmp_ident)),
                    (ArgType::Normal, create_expr_ident(else_tmp_ident)),
                    (ArgType::Normal, create_expr_ident(condition.clone())),
                ],
            ),
            true,
        ));
    }
    assert!(else_temporary_map.is_empty());

    (stmts, tmps)
}

fn convert_to_temporaries(stmts: &[Stmt], ty: &str) -> (Vec<Stmt>, HashMap<Ident, Ident>) {
    let mut result = Vec::new();
    let mut temporary_map = HashMap::new();
    for stmt in stmts {
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
                    // create a temporary
                    let left_ident = &left_path.path.segments[0].ident;
                    let tmp_ident =
                        Ident::new(&format!("__mck_{}_{}", ty, left_ident), left_ident.span());
                    temporary_map.insert(left_ident.clone(), tmp_ident.clone());
                    let mut assign = assign.clone();
                    assign.left = Box::new(create_expr_path(create_path_from_ident(tmp_ident)));

                    let expr = Expr::Assign(assign);
                    result.push(Stmt::Expr(expr.clone(), Some(*semi)));
                }
                _ => panic!("Unexpected expression type: {:?}", stmt),
            },
            _ => panic!("Unexpected statement type: {:?}", stmt),
        }
    }
    (result, temporary_map)
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
            has_leading_colon: false,
            segments: vec![PathRuleSegment::Wildcard],
        },
    ])
}
