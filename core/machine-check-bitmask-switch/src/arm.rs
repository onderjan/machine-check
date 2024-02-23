use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fmt::Write,
};

use num::{BigUint, One, Zero};
use proc_macro2::Span;
use syn::{
    spanned::Spanned, BinOp, Block, Expr, ExprAssign, ExprBinary, ExprIf, ExprParen, ExprPath,
    Ident, LitStr, Local, LocalInit, Pat, PatIdent, Path, Stmt, Token,
};

use crate::{
    util::{convert_bit_length, convert_type, create_number_expr},
    BitmaskArm, BitmaskArmChoice, CareValue, MaskBit,
};

struct ArmStatementCreator {
    scrutinee_expr: Expr,
    something_taken_expr: Expr,

    num_bits: Option<usize>,
    arm_data: Vec<(LitStr, CareValue)>,
    arm_stmts: Vec<Stmt>,
    has_default: bool,
}

pub fn process_arms(
    scrutinee_ident: Ident,
    something_taken_ident: Ident,
    arms: Vec<BitmaskArm>,
) -> (Vec<Stmt>, usize) {
    let mut statement_creator = ArmStatementCreator {
        scrutinee_expr: Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path::from(scrutinee_ident),
        }),
        something_taken_expr: Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path::from(something_taken_ident),
        }),
        num_bits: None,
        arm_data: Vec::new(),
        arm_stmts: Vec::new(),
        has_default: false,
    };

    // process normal arms first, default arms after that
    let mut default_arms = Vec::new();
    for arm in arms {
        if let BitmaskArmChoice::Normal(choice) = arm.choice {
            statement_creator.process_arm(choice, *arm.body);
        } else {
            default_arms.push(arm);
        }
    }

    for default_arm in default_arms {
        let span = default_arm.fat_arrow_token.span();
        statement_creator.process_default_arm(span, *default_arm.body);
    }

    // make sure the arms are disjoint
    for (first_arm_index, first_arm) in statement_creator.arm_data.iter().enumerate() {
        for second_arm in statement_creator.arm_data.iter().skip(first_arm_index + 1) {
            if first_arm.1.intersects(&second_arm.1) {
                panic!(
                    "Arms are not disjoint: {} intersects {}",
                    first_arm.0.value(),
                    second_arm.0.value()
                );
            }
        }
    }

    let Some(num_bits) = statement_creator.num_bits else {
        panic!("There must be at least one non-default arm");
    };

    if !statement_creator.has_default {
        // check full coverage by combining minterms as in the Quine–McCluskey algorithm

        // organize the minterms by the number of care bits
        let mut minterm_map = BTreeMap::<u64, VecDeque<_>>::new();
        for (_lit_str, care_value) in statement_creator.arm_data {
            minterm_map
                .entry(care_value.num_care_bits())
                .or_default()
                .push_back(care_value);
        }

        // combine the smallest minterms in a loop until there are no care bits
        // keep a list of minterms that could not be combined
        let mut uncombinable_minterms = Vec::new();

        while let Some((num_cares, mut smallest_minterms)) = minterm_map.pop_last() {
            // remove the entry with the highest number of care bits (i.e. smallest minterms)
            if num_cares == 0 {
                // OK, full coverage
                continue;
            }
            // we should be able to combine each minterm with another one
            // prefer combining the first minterms first
            while let Some(first_minterm) = smallest_minterms.pop_front() {
                let mut removal_index = None;
                for (second_index, second_minterm) in smallest_minterms.iter().enumerate() {
                    if let Some(combined_minterm) = first_minterm.try_combine(second_minterm) {
                        // we were able to combine the minterms
                        // add the combined one to minterm map and remove the second minterm after breaking
                        let combined_care_bits = combined_minterm.num_care_bits();
                        assert!(combined_care_bits < num_cares);
                        minterm_map
                            .entry(combined_care_bits)
                            .or_default()
                            .push_back(combined_minterm);
                        removal_index = Some(second_index);
                        break;
                    }
                }
                if let Some(removal_index) = removal_index {
                    // OK, we found the second minterm and will remove it
                    smallest_minterms.remove(removal_index);
                } else {
                    // we found a minterm that cannot be combined
                    uncombinable_minterms.push(first_minterm);
                }
            }
        }

        if !uncombinable_minterms.is_empty() {
            let string = uncombinable_minterms
                .iter()
                .fold((String::new(), true), |(mut string, first), item| {
                    if !first {
                        string += ", ";
                    }
                    // string write cannot return an error
                    write!(string, "{}", item).unwrap();
                    (string, false)
                })
                .0;
            panic!("Incomplete coverage, covering only {}", &string);
        }
    }

    (statement_creator.arm_stmts, num_bits)
}

impl ArmStatementCreator {
    fn process_arm(&mut self, choice: LitStr, body: Expr) {
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
                char if char.is_ascii_alphabetic() => MaskBit::Variable(char),
                '0' => MaskBit::Literal(false),
                '1' => MaskBit::Literal(true),
                _ => panic!("Unexpected character '{}'", char),
            };
            // push to front as we need to reverse human-readable
            // to get proper indexing (0 = lowest bit)
            mask_bits.push_front(mask_bit);
        }
        let num_bits = mask_bits.len();
        if let Some(prev_num_bits) = self.num_bits {
            if num_bits != prev_num_bits {
                panic!("Incompatible number of bits");
            }
        } else {
            self.num_bits = Some(num_bits);
        }

        // construct care/value combo, do not care about anything at first
        let mut condition = CareValue {
            num_bits: num_bits as u64,
            care: Zero::zero(),
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
                    // care about this bit and set appropriate value
                    condition.care.set_bit(bit_index as u64, true);
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

        let mask_expr = create_number_expr(&condition.care, num_bits, choice_span);
        let value_expr = create_number_expr(&condition.value, num_bits, choice_span);

        self.arm_data.push((choice, condition));

        // scrutinee & mask == value
        let bitand_expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(self.scrutinee_expr.clone()),
            op: syn::BinOp::BitAnd(Token![&](choice_span)),
            right: Box::new(mask_expr),
        });

        let cond_expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(bitand_expr),
            op: BinOp::Eq(Token![==](choice_span)),
            right: Box::new(value_expr),
        });

        // set something taken
        let mut arm_block_stmts = Vec::new();

        let something_taken_flag_stmt = Stmt::Expr(
            Expr::Assign(ExprAssign {
                attrs: vec![],
                left: Box::new(self.something_taken_expr.clone()),
                eq_token: Token![=](choice_span),
                right: Box::new(create_number_expr(
                    &BigUint::from(1u8),
                    1,
                    self.scrutinee_expr.span(),
                )),
            }),
            Some(Token![;](choice_span)),
        );

        arm_block_stmts.push(something_taken_flag_stmt);

        // construct variables from runs
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

                let run_mask_expr = create_number_expr(&run_mask, num_bits, choice_span);

                // construct run expression (scrutinee & run_mask) >> right_shift
                let mut run_expr = Expr::Binary(ExprBinary {
                    attrs: vec![],
                    left: Box::new(self.scrutinee_expr.clone()),
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
                        create_number_expr(&BigUint::from(right_shift), num_bits, choice_span);
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

            // convert to variable bit length
            let variable_init_expr = variable_init_expr.expect("Mask variable should have init");
            let variable_length = variable_bit_index;
            let variable_init_expr =
                convert_bit_length(variable_init_expr, variable_length, choice_span);

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
                    expr: Box::new(convert_type(
                        variable_init_expr,
                        variable_length,
                        choice_span,
                        false,
                    )),
                    diverge: None,
                }),
                semi_token: Token![;](choice_span),
            });
            arm_block_stmts.push(local_stmt);
        }

        // add arm body after the mask variable initializations
        arm_block_stmts.push(Stmt::Expr(body, Some(Default::default())));

        // construct if expression and add it to arm expressions
        let then_block = Block {
            brace_token: Default::default(),
            stmts: arm_block_stmts,
        };

        let if_expr = ExprIf {
            attrs: vec![],
            if_token: Token![if](choice_span),
            cond: Box::new(cond_expr),
            then_branch: then_block,
            else_branch: None,
        };
        let if_expr_span = if_expr.span();
        self.arm_stmts
            .push(Stmt::Expr(Expr::If(if_expr), Some(Token![;](if_expr_span))));
    }

    fn process_default_arm(&mut self, span: Span, body: Expr) {
        if self.has_default {
            panic!("There can be only one default arm");
        }
        self.has_default = true;

        // take the default arm if nothing was taken
        let cond_expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(self.something_taken_expr.clone()),
            op: BinOp::Eq(Token![==](span)),
            right: Box::new(create_number_expr(&BigUint::from(0u8), 1, span)),
        });

        let then_block = Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(body, Some(Default::default()))],
        };

        let if_expr = ExprIf {
            attrs: vec![],
            if_token: Token![if](span),
            cond: Box::new(cond_expr),
            then_branch: then_block,
            else_branch: None,
        };

        let if_expr_span = if_expr.span();
        self.arm_stmts
            .push(Stmt::Expr(Expr::If(if_expr), Some(Token![;](if_expr_span))));
    }
}
