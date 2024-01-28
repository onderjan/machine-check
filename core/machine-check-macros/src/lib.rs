extern crate proc_macro;
use std::collections::{HashMap, VecDeque};
use std::env::var;

use num::{BigInt, BigUint, One, Zero};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Brace, Comma, FatArrow, Underscore};
use syn::{
    braced, parse_macro_input, BinOp, Block, Expr, ExprBinary, ExprIf, ExprLit, ExprParen,
    ExprPath, Ident, Item, LitInt, LitStr, Local, LocalInit, Pat, PatIdent, Path, PathSegment,
    Stmt, Token,
};

#[proc_macro_attribute]
pub fn machine_description(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let Item::Mod(module) = item else {
        return TokenStream::from(quote_spanned! {
            item.span() =>
            compile_error!("machine_description macro must be used on a module");
        });
    };

    let module_span = module.span();

    let module = match machine_check_machine::process_module(module) {
        Ok(ok) => ok,
        Err(err) => {
            let err_string = err.to_string();
            return TokenStream::from(quote_spanned! {
                module_span =>
                compile_error!(#err_string);
            });
        }
    };

    let expanded = quote! {
        #module
    };
    TokenStream::from(expanded)
}

enum MaskBit {
    Literal(bool),
    Variable(char),
    DontCare,
}

#[derive(Clone, Debug)]
struct MaskValue {
    mask: BigUint,
    value: BigUint,
}

impl MaskValue {
    fn intersects(&self, other: &Self) -> bool {
        // returns true if there is no bit where both masks are 1 and values are different
        let considered_bits = self.mask.bits().min(other.mask.bits());
        for k in 0..considered_bits {
            if self.mask.bit(k) && other.mask.clone().bit(k) {
                // if the values are different, they do not intersect
                if self.value.bit(k) != other.value.bit(k) {
                    return false;
                }
            }
        }
        true
    }
}

#[proc_macro]
pub fn bitmask_switch(stream: TokenStream) -> TokenStream {
    let switch = parse_macro_input!(stream as BitmaskSwitch);

    println!("Bitmask switch: {:?}", switch);

    let scrutinee_span = switch.expr.span();
    // mixed site ident as we do not want the caller to know about it
    let scrutinee_ident = Ident::new("__scrutinee", Span::mixed_site());

    let scrutinee_local = Stmt::Local(Local {
        attrs: vec![],
        let_token: Token![let](scrutinee_span),
        pat: Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: scrutinee_ident.clone(),
            subpat: None,
        }),
        init: Some(LocalInit {
            eq_token: Token![=](scrutinee_span),
            expr: switch.expr,
            diverge: None,
        }),
        semi_token: Token![;](scrutinee_span),
    });

    let scrutinee_expr = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: Path {
            leading_colon: None,
            segments: Punctuated::from_iter([PathSegment {
                ident: scrutinee_ident.clone(),
                arguments: syn::PathArguments::None,
            }]),
        },
    });

    let mut outer_block = Block {
        brace_token: Brace {
            span: switch.brace_token.span,
        },
        stmts: vec![scrutinee_local],
    };

    let mut arm_exprs = Vec::new();

    let mut num_bits = None;

    let mut default_arms = Vec::new();

    let mut mask_values = Vec::new();

    // process normal arms
    for arm in switch.arms {
        let BitmaskArmChoice::Normal(choice) = &arm.choice else {
            default_arms.push(arm);
            continue;
        };
        let mut mask_bits = VecDeque::new();

        let str = choice.value();
        for char in str.chars() {
            if char == '_' {
                // skip underscore
                continue;
            }
            let mask_bit = match char {
                '_' => {
                    // skip underscore
                    continue;
                }
                '-' => MaskBit::DontCare,
                char if unicode_ident::is_xid_start(char) => MaskBit::Variable(char),
                '0' => MaskBit::Literal(false),
                '1' => MaskBit::Literal(true),
                _ => panic!("Unexpected character '{}'", char),
            };
            // push to front as we need to reverse human-readable
            // to get proper indexing (0 = lowest bit)
            mask_bits.push_front(mask_bit);
        }
        if let Some(num_bits) = num_bits {
            if num_bits != mask_bits.len() {
                panic!("Incompatible number of bits");
            }
        } else {
            num_bits = Some(mask_bits.len());
        }

        // construct mask/value combo
        let mut condition = MaskValue {
            mask: Zero::zero(),
            value: Zero::zero(),
        };

        let mut current_run: Option<(char, usize)> = None;
        let mut variable_runs: HashMap<char, Vec<(usize, usize)>> = HashMap::new();

        for (bit_index, mask_bit) in mask_bits.iter().enumerate() {
            // check if we can continue the current run first
            if let Some((run_variable, run_lowest_bit)) = current_run {
                if let MaskBit::Variable(variable) = mask_bit {
                    if *variable == run_variable {
                        // continuing the current run
                        continue;
                    }
                }
                // end the current run
                current_run = None;
                let run_highest_bit = bit_index - 1;
                assert!(run_lowest_bit <= run_highest_bit);
                variable_runs
                    .entry(run_variable)
                    .or_default()
                    .push((run_lowest_bit, run_highest_bit));
            }

            match mask_bit {
                MaskBit::Literal(literal) => {
                    // unmask this bit and set appropriate value
                    condition.mask.set_bit(bit_index as u64, true);
                    condition.value.set_bit(bit_index as u64, *literal);
                }
                MaskBit::Variable(char) => {
                    // start a run
                    current_run = Some((*char, bit_index));
                }
                MaskBit::DontCare => {
                    // do nothing
                }
            }
        }

        // end the current run if it is still going on
        if let Some((run_variable, run_lowest_bit)) = current_run {
            let run_highest_bit = mask_bits.len() - 1;
            assert!(run_lowest_bit <= run_highest_bit);
            variable_runs
                .entry(run_variable)
                .or_default()
                .push((run_lowest_bit, run_highest_bit));
        }

        let choice_span = choice.span();

        let mask_expr = create_number_expr(&condition.mask, choice_span);
        let value_expr = create_number_expr(&condition.value, choice_span);

        mask_values.push(condition);

        // scrutinee & mask == value
        let bitand_expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(scrutinee_expr.clone()),
            op: syn::BinOp::BitAnd(Token![&](choice_span)),
            right: Box::new(mask_expr),
        });

        let cond_expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(bitand_expr),
            op: BinOp::Eq(Token![==](choice_span)),
            right: Box::new(value_expr),
        });

        // construct variables from runs
        let mut arm_block_stmts = Vec::new();
        for (variable, runs) in variable_runs {
            let mut variable_bit_index = 0;
            let mut variable_init_expr = None;

            for (lowest_bit, highest_bit) in runs {
                // mask the value, then shift down
                let run_length = highest_bit - lowest_bit + 1;
                // construct the run-length mask first
                let mut run_mask: BigUint = One::one();
                run_mask <<= run_length;
                run_mask -= 1usize;
                // move it up to obtain run mask
                run_mask <<= lowest_bit;

                // the shift is always to the right, reindexes relative to the variable
                let right_shift = lowest_bit - variable_bit_index;

                let run_mask_expr = create_number_expr(&run_mask, choice_span);

                // construct run expression (scrutinee & run_mask) >> right_shift
                let mut run_expr = Expr::Binary(ExprBinary {
                    attrs: vec![],
                    left: Box::new(scrutinee_expr.clone()),
                    op: BinOp::BitAnd(Token![&](choice_span)),
                    right: Box::new(run_mask_expr),
                });

                // enclose the bit and in parentheses for correct operation precedence
                run_expr = Expr::Paren(ExprParen {
                    attrs: vec![],
                    paren_token: Default::default(),
                    expr: Box::new(run_expr),
                });

                // do not create a right shift operation if unnecessary
                if right_shift != 0 {
                    let right_shift_expr =
                        create_number_expr(&BigUint::from(right_shift), choice_span);
                    run_expr = Expr::Binary(ExprBinary {
                        attrs: vec![],
                        left: Box::new(run_expr),
                        op: BinOp::Shr(Token![>>](choice_span)),
                        right: Box::new(right_shift_expr),
                    });
                    // enclose the right shift operation in parentheses to make operator precedence clearer
                    run_expr = Expr::Paren(ExprParen {
                        attrs: vec![],
                        paren_token: Default::default(),
                        expr: Box::new(run_expr),
                    });
                }

                // bit-or the variable init expression
                // put the current higher-index run on the left for better readability
                if let Some(variable_init_expr_val) = variable_init_expr {
                    variable_init_expr = Some(Expr::Binary(ExprBinary {
                        attrs: vec![],
                        left: Box::new(run_expr),
                        op: BinOp::BitOr(Token![|](choice_span)),
                        right: Box::new(variable_init_expr_val),
                    }));
                } else {
                    variable_init_expr = Some(run_expr);
                }

                // update index within the created variable
                variable_bit_index += run_length;
            }

            let variable_init_expr = variable_init_expr.expect("Mask variable should have init");

            // define and init the local variable
            // this must have call-site hygiene so that we can use the local variable later
            let variable_ident = Ident::new(&variable.to_string(), Span::call_site());
            let variable_pat = Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: variable_ident,
                subpat: None,
            });

            let local_stmt = Stmt::Local(Local {
                attrs: vec![],
                let_token: Token![let](choice_span),
                pat: variable_pat,
                init: Some(LocalInit {
                    eq_token: Token![=](choice_span),
                    expr: Box::new(variable_init_expr),
                    diverge: None,
                }),
                semi_token: Token![;](choice_span),
            });
            arm_block_stmts.push(local_stmt);
        }

        // add arm body after the mask variable initializations
        arm_block_stmts.push(Stmt::Expr(*arm.body, None));

        // construct if expression and add it to arm expressions
        let then_block = Block {
            brace_token: Default::default(),
            stmts: arm_block_stmts,
        };

        let if_expr = Expr::If(ExprIf {
            attrs: vec![],
            if_token: Token![if](choice_span),
            cond: Box::new(cond_expr),
            then_branch: then_block,
            else_branch: None,
        });

        arm_exprs.push(if_expr);
    }

    // make sure the arms are disjoint
    for i in 0..mask_values.len() {
        for j in i + 1..mask_values.len() {
            if mask_values[i].intersects(&mask_values[j]) {
                panic!("Arms are not disjoint");
            }
        }
    }

    // process default arm
    let mut has_default = false;
    for default_arm in default_arms {
        let BitmaskArmChoice::Default(_underscore) = default_arm.choice else {
            panic!("Unexpected non-default arm");
        };
        if has_default {
            panic!("There can be only one default arm");
        }
        has_default = true;

        // add default body to the end
        arm_exprs.push(*default_arm.body);
    }

    if !has_default {
        // TODO: check full coverage using the Quine–McCluskey algorithm
        panic!("There currently must be a default arm");
    }

    let mut chain_expr = None;

    // convert arm expression to if-else-chain, construct by iterating in reverse
    for arm_expr in arm_exprs.into_iter().rev() {
        let new_expr = if let Some(chain_expr) = chain_expr.take() {
            // set the current chain to the else block of the arm condition
            let Expr::If(mut arm_if_expr) = arm_expr else {
                panic!("Every arm expression except possibly the last should be if expression");
            };
            arm_if_expr.else_branch = Some((
                Token![else](arm_if_expr.if_token.span),
                Box::new(chain_expr),
            ));
            Expr::If(arm_if_expr)
        } else {
            arm_expr
        };
        chain_expr = Some(new_expr);
    }

    let chain_expr = chain_expr.expect("There must be at least one arm");

    // add chain to outer block
    outer_block.stmts.push(Stmt::Expr(chain_expr, None));

    let expanded = quote! {
        #outer_block
    };
    println!("Expanded: {}", expanded);

    TokenStream::from(expanded)
}

fn create_number_expr(num: &BigUint, span: Span) -> Expr {
    Expr::Lit(ExprLit {
        attrs: vec![],
        lit: syn::Lit::Int(LitInt::new(&num.to_string(), span)),
    })
}

#[derive(Debug, Clone)]
enum BitmaskArmChoice {
    Normal(LitStr),
    Default(Underscore),
}

impl Parse for BitmaskArmChoice {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![_]) {
            Ok(Self::Default(input.parse()?))
        } else {
            Ok(Self::Normal(input.parse()?))
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BitmaskArm {
    choice: BitmaskArmChoice,
    fat_arrow_token: FatArrow,
    body: Box<Expr>,
    comma: Option<Comma>,
}

impl Parse for BitmaskArm {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // parse similarly to syn Arm
        // https://docs.rs/syn/latest/src/syn/expr.rs.html#2833
        let choice = input.parse()?;
        let fat_arrow_token = input.parse()?;
        let body = input.parse()?;

        // inspired by requires_terminator
        // https://docs.rs/syn/latest/src/syn/expr.rs.html#916-958
        let comma_needed = !matches!(
            body,
            Expr::If(_)
                | Expr::Match(_)
                | Expr::Block(_)
                | Expr::Unsafe(_)
                | Expr::While(_)
                | Expr::Loop(_)
                | Expr::ForLoop(_)
                | Expr::TryBlock(_)
                | Expr::Const(_)
        );

        let comma = if comma_needed && !input.is_empty() {
            Some(input.parse()?)
        } else {
            input.parse()?
        };

        Ok(BitmaskArm {
            choice,
            fat_arrow_token,
            body: Box::new(body),
            comma,
        })
    }
}

#[derive(Debug, Clone)]
struct BitmaskSwitch {
    expr: Box<Expr>,
    brace_token: Brace,
    arms: Vec<BitmaskArm>,
}

impl Parse for BitmaskSwitch {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // parse similarly to syn ExprMatch
        // https://docs.rs/syn/latest/src/syn/expr.rs.html#2225
        // no attributes, start with scrutinee
        let expr = Expr::parse_without_eager_brace(input)?;

        let inside_braces;
        let brace_token = braced!(inside_braces in input);

        let mut arms = Vec::new();
        while !inside_braces.is_empty() {
            arms.push(inside_braces.call(BitmaskArm::parse)?);
        }

        Ok(BitmaskSwitch {
            expr: Box::new(expr),
            brace_token,
            arms,
        })
    }
}
