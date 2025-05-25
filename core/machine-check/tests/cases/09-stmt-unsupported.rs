#[machine_check_macros::machine_description]
mod machine {
    struct A {}

    impl A {
        fn function() -> ::machine_check::Bitvector<32> {
            // statements
            fn item_inside_fn() {}
            statement_macro! {}
            {
                block_statement_with_result
            };

            // locals
            //let (unsupported_local_pattern) = 0; // pre-rejected by use
            let diverging_let = 0 else {};
            let ref x = y;
            let a @ b = c;

            // statement expression
            break;

            // assignment expression
            machine_check::non_ident_left_path = 0;
            (non_ident_left_expression) = 0;
            (non_ident_left_base).index = 0;
            //base.0 = 0; // pre-rejected by use
            0 = 0;

            // macro expression
            unknown_macro!();
            ::std::panic!(non_literal_arg);

            ::machine_check::Bitvector::<32>::new(0)
        }
    }
}

fn main() {}
