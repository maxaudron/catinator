use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};

use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    LitStr, Path, Token,
};

pub mod privmsg;

pub trait IrcItem {
    fn to_call(&self) -> proc_macro2::TokenStream;
    fn help(&self) -> String;
}

pub struct Items {
    pub inner: Vec<Item>,
}

impl Parse for Items {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items: Vec<Item> = Vec::new();

        while !input.is_empty() {
            if input.peek(syn::Ident) && input.peek2(syn::token::Paren) || input.peek(Token![async])
            {
                items.push(input.parse()?)
            } else {
                return Err(input.error("line was not of expected format"));
            }
        }

        Ok(Self { inner: items })
    }
}

pub enum Item {
    Command(Command),
    Hook(Hook),
    Matcher(Matcher),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut asyn = false;
        if input.peek(Token![async]) {
            asyn = true;
            let _: Token![async] = input.parse()?;
        }

        if input.peek(syn::Ident) {
            // parses the identifier to determine the type
            let item: Ident = input.parse()?;
            match item.to_string().as_str() {
                "command" => input.parse().map(|mut i: Command| {
                    i.asyn = asyn;
                    Item::Command(i)
                }),
                "hook" => input.parse().map(|mut i: Hook| {
                    i.asyn = asyn;
                    Item::Hook(i)
                }),
                "matcher" => input.parse().map(|mut i: Matcher| {
                    i.asyn = asyn;
                    Item::Matcher(i)
                }),
                _ => Err(input.error(format!(
                    "expected one of: command, hook or matcher not {}",
                    item.to_string()
                ))),
            }
        } else {
            Err(input.error("wrong type"))
        }
    }
}

pub struct Command {
    pub asyn: bool,
    pub name: LitStr,
    pub description: LitStr,
    pub function: Function,
}

impl IrcItem for Command {
    fn to_call(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let function = &self.function;

        let call = if self.asyn {
            quote! {
                #function(&bot, message.clone()).await
            }
        } else {
            quote! {
                #function(&bot, message.clone())
            }
        };

        quote! {
            if #name == rest {
                debug!(target: "command", "{} with {:?}", #name, message);
                let result = #call;

                if let Err(err) = result {
                    tracing::warn!("error in matcher: {:?}: {:?}", #name, err)
                }
            }
        }
    }

    fn help(&self) -> String {
        format!("  {}: {}", self.name.value(), self.description.value())
    }
}

impl Parse for Command {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let mut _token: Token![,];

        if input.peek(Token![,]) {
            _token = input.parse()?
        }

        let name = content.parse()?;
        _token = content.parse()?;
        let description = content.parse()?;
        _token = content.parse()?;
        let function = content.parse()?;

        Ok(Self {
            asyn: false,
            name,
            description,
            function,
        })
    }
}

pub struct Hook {
    pub asyn: bool,
    pub name: LitStr,
    pub description: LitStr,
    pub kind: Ident,
    pub function: Function,
}

impl IrcItem for Hook {
    fn to_call(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let kind = &self.kind;
        let kind_str = &self.kind.to_string();
        let function = &self.function;

        let call = if self.asyn {
            quote! {
                #function(&bot, message.clone()).await
            }
        } else {
            quote! {
                #function(&bot, message.clone())
            }
        };

        quote! {
            if let Command::#kind(..) = &command {
                debug!(target: "hook", "{} of kind {} with {:?}", #name, #kind_str, message);
                let result = #call;

                if let Err(err) = result {
                    tracing::warn!("error in matcher: {:?}: {:?}", #name, err)
                }
            }
        }
    }

    fn help(&self) -> String {
        format!("  {}: {}", self.name.value(), self.description.value())
    }
}

impl Parse for Hook {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let mut _token: Token![,];

        if input.peek(Token![,]) {
            _token = input.parse()?;
        }

        let name = content.parse()?;
        _token = content.parse()?;
        let description = content.parse()?;
        _token = content.parse()?;
        let kind: Ident = content.parse()?;
        match kind.to_string().as_str() {
            "PASS" | "NICK" | "USER" | "OPER" | "UserMODE" | "SERVICE" | "QUIT" | "SQUIT"
            | "JOIN" | "PART" | "ChannelMODE" | "TOPIC" | "NAMES" | "LIST" | "INVITE" | "KICK"
            | "PRIVMSG" | "NOTICE" | "MOTD" | "LUSERS" | "VERSION" | "STATS" | "LINKS" | "TIME"
            | "CONNECT" | "TRACE" | "ADMIN" | "INFO" | "SERVLIST" | "SQUERY" | "WHO" | "WHOIS"
            | "WHOWAS" | "KILL" | "PING" | "PONG" | "ERROR" | "AWAY" | "REHASH" | "DIE"
            | "RESTART" | "SUMMON" | "USERS" | "WALLOPS" | "USERHOST" | "ISON" | "SAJOIN"
            | "SAMODE" | "SANICK" | "SAPART" | "SAQUIT" | "NICKSERV" | "CHANSERV" | "OPERSERV"
            | "BOTSERV" | "HOSTSERV" | "MEMOSERV" | "CAP" | "AUTHENTICATE" | "ACCOUNT"
            | "METADATA" | "MONITOR" | "BATCH" | "CHGHOST" | "Response" | "Raw" => (),
            _ => {
                return Err(content.error(format!(
                    "expected one of: PASS, NICK, USER, OPER, UserMODE, SERVICE, QUIT, SQUIT
           , JOIN, PART, ChannelMODE, TOPIC, NAMES, LIST, INVITE, KICK
           , PRIVMSG, NOTICE, MOTD, LUSERS, VERSION, STATS, LINKS, TIME
           , CONNECT, TRACE, ADMIN, INFO, SERVLIST, SQUERY, WHO, WHOIS
           , WHOWAS, KILL, PING, PONG, ERROR, AWAY, REHASH, DIE
           , RESTART, SUMMON, USERS, WALLOPS, USERHOST, ISON, SAJOIN
           , SAMODE, SANICK, SAPART, SAQUIT, NICKSERV, CHANSERV, OPERSERV
           , BOTSERV, HOSTSERV, MEMOSERV, CAP, AUTHENTICATE, ACCOUNT
           , METADATA, MONITOR, BATCH, CHGHOST, Response, Raw not {}",
                    kind.to_string()
                )))
            }
        }

        _token = content.parse()?;
        let function = content.parse()?;

        Ok(Self {
            asyn: false,
            name,
            description,
            kind,
            function,
        })
    }
}

pub struct Matcher {
    pub asyn: bool,
    pub name: LitStr,
    pub description: LitStr,
    pub matcher: LitStr,
    pub function: Function,
}

impl IrcItem for Matcher {
    fn to_call(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let function = &self.function;

        let ident = Ident::new(&name.value(), Span::call_site());

        let call = if self.asyn {
            quote! {
                #function(&bot, message.clone()).await
            }
        } else {
            quote! {
                #function(&bot, message.clone())
            }
        };

        quote! {
            if #ident.is_match(text) {
                debug!(target: "matcher", "{} with {:?}", #name, message);
                let result = #call;

                if let Err(err) = result {
                    tracing::warn!("error in matcher: {:?}: {:?}", #name, err)
                }
            }
        }
    }

    fn help(&self) -> String {
        format!(
            "  {} ({}):  {}",
            self.name.value(),
            self.matcher.value(),
            self.description.value()
        )
    }
}

impl Parse for Matcher {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let mut _token: Token![,];

        if input.peek(Token![,]) {
            _token = input.parse()?;
        }

        let name = content.parse()?;
        _token = content.parse()?;
        let description = content.parse()?;
        _token = content.parse()?;
        let matcher = content.parse()?;
        _token = content.parse()?;
        let function = content.parse()?;

        Ok(Self {
            asyn: false,
            name,
            description,
            matcher,
            function,
        })
    }
}

pub enum Function {
    Path(Path),
    Expr(Punctuated<Ident, Token![.]>),
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Function::Path(v) => *tokens = v.into_token_stream(),
            Function::Expr(v) => *tokens = v.into_token_stream(),
        }
    }
}

impl Parse for Function {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek2(Token![::]) {
            Ok(Function::Path(input.parse()?))
        } else if input.peek2(Token![.]) {
            Ok(Function::Expr(input.parse_terminated(Ident::parse, Token![.])?))
        } else {
            Err(input.error("did not find path or dotted"))
        }
    }
}
