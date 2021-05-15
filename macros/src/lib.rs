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

        let config_path = env::var("CATINATOR_CONFIG").unwrap_or("config.toml".to_string());
        info!("starting bot with config file {}", config_path);
        let mut bot = Bot::new(&config_path).await.unwrap();

        if bot.config.server.sasl {
            info!("initializing sasl");
            bot.sasl_init().await.unwrap()
        }

        #(#matchers_regex)*

        info!("starting main event loop");
        let mut stream = bot.irc_client.stream().unwrap();

        while let Some(message) = stream.next().await.transpose().unwrap() {
            trace!("{:?}", message);

            let command = message.clone().command;

            #(#hooks)*

            match &command {
                Command::PRIVMSG(_target, text) => {
                    let mut word = text.split_ascii_whitespace().next().unwrap().chars();
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
