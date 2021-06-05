use syn::{Expr, Token, parse::{Parse, ParseStream}};

pub struct Item {
    pub msg: Expr,
    pub tok2: Token![,],
    pub func: Expr,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            msg: input.parse()?,
            tok2: input.parse()?,
            func: input.parse()?,
        })
    }
}
