use syn::{
    visit_mut::{self, VisitMut},
    Block, ExprAssign, ExprBlock, ExprCall, Ident, ItemStruct, Path, Stmt,
};
use syn_path::path;

use crate::{
    support::field_manipulate,
    util::{
        create_expr_ident, create_expr_path, extract_else_token_block, extract_expr_ident,
        path_matches_global_names,
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

    for item in abstract_machine.items.iter_mut() {
        Visitor().visit_item_mut(item);
    }

    // add field-manipulate to items
    field_manipulate::apply_to_items(&mut abstract_machine.items, "abstr")?;

    Ok(abstract_machine)
}

struct Visitor();
impl VisitMut for Visitor {
    fn visit_item_struct_mut(&mut self, s: &mut ItemStruct) {
        // add default derive attributes to the structs
        // that easily allow us to make unknown inputs/states
        s.attrs
            .push(generate_derive_attribute(quote!(::std::default::Default)));
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // propagate first
        visit_mut::visit_expr_mut(self, expr);
        // then convert
        convert_expr(expr);
    }
}

fn convert_expr(expr: &mut Expr) {
    let Expr::If(expr_if) = expr else {
        return;
    };
    let Expr::Call(cond_expr_call) = expr_if.cond.as_mut() else {
        return;
    };
    let Expr::Path(cond_expr_path) = cond_expr_call.func.as_mut() else {
        return;
    };
    if cond_expr_path.path != path!(::mck::abstr::Test::into_bool) {
        return;
    }
    if cond_expr_call.args.len() != 1 {
        panic!("Expected into_bool call to have exactly one argument");
    }

    let if_token = expr_if.if_token;
    let condition = extract_expr_ident(&cond_expr_call.args[0]).expect("Condition should be ident");

    let then_block = &expr_if.then_branch;

    // create a new condition in the else block
    let (else_token, else_block) =
        extract_else_token_block(&expr_if.else_branch).expect("Expected if with else block");

    // split into a block that contains two if statements with then branch for each branch of original:
    // 1. can be true
    // 2. can be false
    // in then branch, retain Taken within the statements, but eliminate NotTaken
    // in else branch, convert the Taken from then branch to NotTaken

    let can_be_true_if = create_branch_if(
        path!(::mck::abstr::Test::can_be_true),
        then_block,
        condition,
        cond_expr_call,
        if_token,
        else_token,
    );

    let can_be_true_else = create_branch_if(
        path!(::mck::abstr::Test::can_be_false),
        else_block,
        condition,
        cond_expr_call,
        if_token,
        else_token,
    );

    let outer_expr = Expr::Block(ExprBlock {
        attrs: vec![],
        label: None,
        block: Block {
            brace_token: Default::default(),
            stmts: vec![
                Stmt::Expr(Expr::If(can_be_true_if), Some(Default::default())),
                Stmt::Expr(Expr::If(can_be_true_else), Some(Default::default())),
            ],
        },
    });

    *expr = outer_expr;
}

fn create_branch_if(
    cond_path: Path,
    taken_block: &Block,
    condition: &Ident,
    cond_expr_call: &ExprCall,
    if_token: syn::token::If,
    else_token: syn::token::Else,
) -> ExprIf {
    let can_be_true_cond = Expr::Call(ExprCall {
        attrs: cond_expr_call.attrs.clone(),
        func: Box::new(create_expr_path(cond_path)),
        paren_token: cond_expr_call.paren_token,
        args: cond_expr_call.args.clone(),
    });

    let mut taken_branch_block = taken_block.clone();
    let not_taken_branch_block = process_taken_branch_block(&mut taken_branch_block, condition);

    ExprIf {
        attrs: vec![],
        if_token,
        cond: Box::new(can_be_true_cond),
        then_branch: taken_branch_block,
        else_branch: Some((
            else_token,
            Box::new(Expr::Block(ExprBlock {
                attrs: vec![],
                label: None,
                block: not_taken_branch_block,
            })),
        )),
    }
}

fn process_taken_branch_block(taken_block: &mut Block, condition: &Ident) -> Block {
    let mut taken_stmts = Vec::new();
    let mut not_taken_stmts = Vec::new();
    for mut stmt in taken_block.stmts.drain(..) {
        let mut retain = true;
        if let Stmt::Expr(Expr::Assign(expr_assign), Some(semi)) = &mut stmt {
            if let Expr::Call(expr_call) = expr_assign.right.as_mut() {
                if let Expr::Path(expr_path) = expr_call.func.as_mut() {
                    if path_matches_global_names(
                        &expr_path.path,
                        &["mck", "forward", "PhiArg", "NotTaken"],
                    ) {
                        // do not retain
                        retain = false;
                    }
                    if path_matches_global_names(
                        &expr_path.path,
                        &["mck", "forward", "PhiArg", "Taken"],
                    ) {
                        // retain as MaybeTaken, add condition
                        let last_ident = &mut expr_path.path.segments[3].ident;
                        *last_ident = Ident::new("MaybeTaken", last_ident.span());
                        expr_call.args.push(create_expr_ident(condition.clone()));

                        // retain, but also add as not taken to the else block
                        let mut not_taken_expr_path = expr_path.clone();
                        let not_taken_last_ident = &mut not_taken_expr_path.path.segments[3].ident;
                        *not_taken_last_ident = Ident::new("NotTaken", not_taken_last_ident.span());
                        // not taken has no arguments
                        let not_taken_expr_call = ExprCall {
                            attrs: expr_call.attrs.clone(),
                            func: Box::new(Expr::Path(not_taken_expr_path)),
                            paren_token: expr_call.paren_token,
                            args: Default::default(),
                        };
                        let not_taken_expr_assign = ExprAssign {
                            attrs: expr_assign.attrs.clone(),
                            left: expr_assign.left.clone(),
                            eq_token: expr_assign.eq_token,
                            right: Box::new(Expr::Call(not_taken_expr_call)),
                        };
                        let not_taken_stmt =
                            Stmt::Expr(Expr::Assign(not_taken_expr_assign), Some(*semi));
                        not_taken_stmts.push(not_taken_stmt);
                    }
                }
            }
        };
        if retain {
            taken_stmts.push(stmt);
        }
    }
    taken_block.stmts = taken_stmts;

    Block {
        brace_token: taken_block.brace_token,
        stmts: not_taken_stmts,
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
