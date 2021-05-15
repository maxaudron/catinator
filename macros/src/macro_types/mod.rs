use proc_macro2::{Ident, Span};
use quote::quote;

use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Path,
};
use syn::{LitStr, Token};

pub trait IrcItem {
    fn to_call(&self) -> proc_macro2::TokenStream;
    fn help(&self) -> String;
}

#[derive(Debug)]
pub struct Items {
    pub inner: Vec<Item>,
}

impl Parse for Items {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items: Vec<Item> = Vec::new();

        while !input.is_empty() {
            if input.peek(syn::Ident) && input.peek2(syn::token::Paren) {
                items.push(input.parse()?)
            } else {
                return Err(input.error("line was not of expected format"));
            }
        }

        Ok(Self { inner: items })
    }
}

#[derive(Debug)]
pub enum Item {
    Command(Command),
    Hook(Hook),
    Matcher(Matcher),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            let item: Ident = input.parse()?;
            match item.to_string().as_str() {
                "command" => input.parse().map(Item::Command),
                "hook" => input.parse().map(Item::Hook),
                "matcher" => input.parse().map(Item::Matcher),
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

#[derive(Debug)]
pub struct Command {
    pub name: LitStr,
    pub description: LitStr,
    pub function: Path,
}

impl IrcItem for Command {
    fn to_call(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let function = &self.function;

        quote! {
            if #name == rest {
                debug!(target: "command", "{} with {:?}", #name, message);
                let result = #function(message.clone());
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
            name,
            description,
            function,
        })
    }
}

#[derive(Debug)]
pub struct Hook {
    pub name: LitStr,
    pub description: LitStr,
    pub kind: Ident,
    pub function: Path,
}

impl IrcItem for Hook {
    fn to_call(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let kind = &self.kind;
        let kind_str = &self.kind.to_string();
        let function = &self.function;

        quote! {
            if let Command::#kind(..) = &command {
                debug!(target: "hook", "{} of kind {} with {:?}", #name, #kind_str, message);
                let result = #function(&bot, message.clone());
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
            name,
            description,
            kind,
            function,
        })
    }
}

#[derive(Debug)]
pub struct Matcher {
    pub name: LitStr,
    pub description: LitStr,
    pub matcher: LitStr,
    pub function: Path,
}

impl IrcItem for Matcher {
    fn to_call(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let function = &self.function;

        let ident = Ident::new(&name.value(), Span::call_site());

        quote! {
            if #ident.is_match(text) {
                debug!(target: "matcher", "{} with {:?}", #name, message);
                let result = #function(&bot, message.clone());
            }
        }
    }

    fn help(&self) -> String {
        format!("  {} ({}):  {}", self.name.value(), self.matcher.value(), self.description.value())
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
            name,
            description,
            matcher,
            function,
        })
    }
}
