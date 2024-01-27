extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::token::{Brace, Comma, FatArrow, Underscore};
use syn::{braced, parse_macro_input, Expr, Item, LitStr, Token};

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

#[proc_macro]
pub fn bitmask_switch(stream: TokenStream) -> TokenStream {
    let switch = parse_macro_input!(stream as BitmaskSwitch);

    println!("Bitmask switch: {:?}", switch);
    let expanded = quote! {
        // ...
    };
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
