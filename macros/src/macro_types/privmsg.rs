use syn::{
    parse::{Parse, ParseStream},
    Expr, Token,
};

pub struct Item {
    pub msg: Expr,
    pub _tok2: Token![,],
    pub func: Expr,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            msg: input.parse()?,
            _tok2: input.parse()?,
            func: input.parse()?,
        })
    }
}
