extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;

use syn::parse_macro_input;

mod macro_types;

use macro_types::*;

fn generate_help(items: &Items) -> proc_macro2::TokenStream {
    let command_help = items.inner.iter().filter_map(|x| {
        if let Item::Command(command) = x {
            let help = command.help();
            Some(quote! {
                bot.send_notice(target, #help).unwrap();
            })
        } else {
            None
        }
    });

    let matcher_help = items.inner.iter().filter_map(|x| {
        if let Item::Matcher(matcher) = x {
            let help = matcher.help();
            Some(quote! {
                bot.send_notice(target, #help).unwrap();
            })
        } else {
            None
        }
    });

    let hook_help = items.inner.iter().filter_map(|x| {
        if let Item::Hook(hook) = x {
            let help = hook.help();
            Some(quote! {
                bot.send_notice(target, #help).unwrap();
            })
        } else {
            None
        }
    });

    let gen = quote! {
        let target = message.source_nickname().unwrap();

        bot.send_notice(target, "COMMANDS:").unwrap();
        #(#command_help)*

        bot.send_notice(target, "MATCHERS:").unwrap();
        #(#matcher_help)*

        bot.send_notice(target, "HOOKS:").unwrap();
        #(#hook_help)*
    };
    gen.into()
}

/// Main entrypoint to the bot
///
/// ```no_run
/// # extern crate tokio;
/// # use macros::catinator;
/// # use anyhow::Result;
/// #
/// # fn function(bot: &catinator::Bot, msg: irc::client::prelude::Message) -> Result<()> {
/// #   Ok(())
/// # }
/// #
/// #[tokio::main]
/// async fn main() {
///   catinator!(
///     hook("name", "A short description", PRIVMSG, function)
///     command("name", "A short description", function)
///     matcher("name", "A short description", r"^\[.*?\]$", function)
///   );
/// }
/// ```
///
/// # Functions
/// All the functions share a similar pattern,
/// The first two arguments are the name and description respectively,
/// the last argument is the function that gets executed.
///
/// The function must of of the following type:
/// ```
/// fn hook(bot: &catinator::Bot, msg: irc::client::prelude::Message) -> anyhow::Result<()> {
///    Ok(())
/// }
/// ```
///
/// ## async
/// You can run async functions natively by prepending your function
/// hooks etc. with the async keyword.
///
/// ```ignore
/// async hook("name", "description", COMMAND, function)
/// ```
///
/// ## hook
/// Hooks execute a function when a specific IRC Command is received,
/// this allows for great flexibility in hooking into IRC for Authentication and the likes.
///
/// ```ignore
/// hook("name", "description", COMMAND, function)
/// ```
///
/// PRIVMSG is an IRC Command like PRIVMSG or AUTHENTICATE
/// Any of the enum variants of [the irc crate](https://docs.rs/irc/0.15.0/irc/client/prelude/enum.Command.html)
/// should work.
///
/// ## command
/// A Command is command that can be executed in any PRIVMSG and is
/// prefixed with the prefix configured in the config.toml file
///
/// ```ignore
/// command("name", "description", function)
/// ```
/// Would be ":name <whatever>" in an irc channel or private message.
///
/// ## matcher
/// A matcher matches on a PRIVMSG using regex.
///
/// ```ignore
/// matcher("name", "description", r"regex", function)
/// ```
///
/// The [regex crate](https://docs.rs/regex) is used for matching, see it's documentation for details.
///
#[proc_macro]
pub fn catinator(tokens: TokenStream) -> TokenStream {
    let items = parse_macro_input!(tokens as Items);

    let hooks = items.inner.iter().filter_map(|x| {
        if let Item::Hook(hook) = x {
            Some(hook.to_call())
        } else {
            None
        }
    });

    let commands = items.inner.iter().filter_map(|x| {
        if let Item::Command(command) = x {
            Some(command.to_call())
        } else {
            None
        }
    });

    let matchers = items.inner.iter().filter_map(|x| {
        if let Item::Matcher(command) = x {
            Some(command.to_call())
        } else {
            None
        }
    });

    let matchers_regex = items.inner.iter().filter_map(|x| {
        if let Item::Matcher(matcher) = x {
            let name = &matcher.name;
            let regex = &matcher.matcher;

            let ident = Ident::new(&name.value(), Span::call_site());

            Some(quote! {
                let #ident = regex::Regex::new(#regex).unwrap();
            })
        } else {
            None
        }
    });

    let help = generate_help(&items);

    let gen = quote! {
        use std::env;

        use futures::prelude::*;
        use tracing::{info, debug, trace};

        use irc::client::prelude::*;
        use catinator::Bot;

        #(#matchers_regex)*

        info!("starting main event loop");
        let mut stream = bot.irc_client.stream().unwrap();

        while let Some(message) = stream.next().await.transpose().unwrap() {
            trace!("{:?}", message);

            let command = message.clone().command;

            #(#hooks)*

            match &command {
                Command::PRIVMSG(_target, text) => {
                    let mut word = match text.split_ascii_whitespace().next() {
                        Some(word) => word.chars(),
                        None => continue,
                    };
                    let prefix = word.next().unwrap();
                    let rest: String = word.collect();

                    if prefix == bot.config.settings.prefix {
                        if "help" == rest {
                            #help
                        }

                        #(#commands)*
                    } else {
                        #(#matchers)*
                    }
                }
                _ => (),
            }
        }
    };

    return gen.into();
}

/// Match on a privmsg and execute the function block on it
///
/// # Examples
/// ```
/// # use anyhow::Result;
/// # use irc::client::prelude::*;
/// # use macros::privmsg;
/// #
/// # pub fn hook(bot: &catinator::Bot, msg: Message) -> Result<()> {
/// privmsg!(msg, {
///     bot.send_privmsg(
///         msg.response_target().unwrap(),
///         "bla",
///     )?;
/// })
/// # }
/// ```
#[proc_macro]
pub fn privmsg(tokens: TokenStream) -> TokenStream {
    use crate::macro_types::privmsg::Item;
    let item = parse_macro_input!(tokens as Item);

    let msg = item.msg;
    let func = item.func;

    let gen = quote! {
        match &#msg.command {
            Command::PRIVMSG(target, text) => {
                #func

                Ok(())
            }
            _ => Ok(()),
        }
    };
    return gen.into();
}
