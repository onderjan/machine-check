#[machine_check_macros::machine_description]
mod machine {
    struct A {}

    impl A {
        fn expressions() -> u32 {
            // field
            a = x.0;
            a = nonident::path.0;
            // struct
            a = Q { ..b };
            a = Q { 0: b };
            // do not test this as it also prints an error
            // that usage of qualified paths in this context is experimental
            // and may change between Rust compiler versions
            /* a = <A as B>::Q {}; */
            // dereference
            a = &(b);
            a = &b.0;
            a = &(nonident::path);
            // index
            a = (b)[10];
            a = b.0[10];
            a = nonident::path[0];
            a = b[nonident::path];

            0
        }

        fn operators() -> u32 {
            // unary operators
            a = *b;
            // binary operators
            a = a && b;
            a = a || b;

            a = b += c;
            a = b -= c;
            a = b *= c;
            a = b /= c;
            a = b %= c;

            a = b ^= c;
            a = b &= c;
            a = b |= c;

            a = b <<= c;
            a = b >>= c;

            0
        }
    }
}

fn main() {}
