extern crate proc_macro;
use std::collections::VecDeque;

use num::{BigInt, BigUint, One, Zero};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Brace, Comma, FatArrow, Underscore};
use syn::{
    braced, parse_macro_input, BinOp, Block, Expr, ExprBinary, ExprIf, ExprLit, ExprPath, Ident,
    Item, LitInt, LitStr, Local, LocalInit, Pat, PatIdent, Path, PathSegment, Stmt, Token,
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
    let scrutinee_ident = Ident::new("__scrutinee", Span::call_site());

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

    let mut outer_block = Block {
        brace_token: Brace {
            span: switch.brace_token.span,
        },
        stmts: vec![scrutinee_local],
    };

    // TODO: check that the arms are disjoint

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
        let one: BigUint = One::one();
        let mut mask_value = MaskValue {
            mask: Zero::zero(),
            value: Zero::zero(),
        };
        for (bit_index, mask_bit) in mask_bits.iter().enumerate() {
            if let MaskBit::Literal(literal) = mask_bit {
                // unmask this bit and set appropriate value
                mask_value.mask.set_bit(bit_index as u64, true);
                mask_value.value.set_bit(bit_index as u64, *literal);
            } else {
                // don't cares and variables are masked
            }
        }

        let choice_span = choice.span();

        let mask_expr = Expr::Lit(ExprLit {
            attrs: vec![],
            lit: syn::Lit::Int(LitInt::new(&mask_value.mask.to_string(), choice_span)),
        });
        let value_expr = Expr::Lit(ExprLit {
            attrs: vec![],
            lit: syn::Lit::Int(LitInt::new(&mask_value.value.to_string(), choice_span)),
        });

        mask_values.push(mask_value);

        // scrutinee & mask == value
        let bitand_expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter([PathSegment {
                        ident: scrutinee_ident.clone(),
                        arguments: syn::PathArguments::None,
                    }]),
                },
            })),
            op: syn::BinOp::BitAnd(Token![&](choice_span)),
            right: Box::new(mask_expr),
        });

        let cond_expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(bitand_expr),
            op: BinOp::Eq(Token![==](choice_span)),
            right: Box::new(value_expr),
        });

        let then_block = Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(*arm.body, None)],
        };
        // TODO: declare and initialize variables

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
        panic!("There currently must be a default arm");
    }

    // TODO: process default arm

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
