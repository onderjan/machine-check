use syn::{
    braced,
    parse::Parse,
    token::{Brace, Comma, FatArrow, Underscore},
    Expr, LitStr, Token,
};

#[derive(Debug, Clone)]
pub enum BitmaskArmChoice {
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
pub struct BitmaskArm {
    pub choice: BitmaskArmChoice,
    pub fat_arrow_token: FatArrow,
    pub body: Box<Expr>,
    pub comma: Option<Comma>,
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
pub struct BitmaskSwitch {
    pub expr: Box<Expr>,
    pub brace_token: Brace,
    pub arms: Vec<BitmaskArm>,
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
