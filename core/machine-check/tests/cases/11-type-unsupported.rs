#[machine_check_macros::machine_description]
mod machine {
    struct A {
        a: ::machine_check::UnsupportedType,
        b: [ArrayElementType; 1],
        c: &mut u32,
    }

    impl A {
        fn a() -> ::machine_check::UnsupportedType {}
        fn b() -> [ArrayElementType; 2] {}
        fn c() -> &mut u32 {}

        fn d(&mut self) -> u32 {}

        fn e(a: ::machine_check::UnsupportedType) -> u32 {}
        fn f(a: [ArrayElementType; 1]) -> u32 {}
        fn g(a: &mut u32) -> u32 {}

        fn h() -> u32 {
            let a: ::machine_check::UnsupportedType;
            let b: [ArrayElementType; 1];
            let c: &mut u32;

            let d: <A as B>::Q;

            0
        }
    }
}

fn main() {}
