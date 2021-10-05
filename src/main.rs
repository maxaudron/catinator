#[tokio::main]
async fn main() {
    use catinator::catinator;

    tracing_subscriber::fmt::init();

    let mut sed = catinator::hooks::sed::Sed::new();

    catinator![
        hook(
            "sasl",
            "Handle Authentication.",
            AUTHENTICATE,
            catinator::hooks::sasl
        ),
        hook(
            "sed_log",
            "Log messages for use with sed replace, max 10k lines.",
            PRIVMSG,
            sed.log
        ),
        async hook(
            "url_preview",
            "Send preview of website",
            PRIVMSG,
            catinator::hooks::url::url_preview
        )
        matcher(
            "shifty_eyes",
            ">.>",
            r"^\S{3}$",
            catinator::hooks::shifty_eyes
        ),
        matcher(
            "replace",
            "sed style replace with regex support. i/g/U/x sed flags available",
            r"^s/",
            sed.replace
        ),
        matcher(
            "intensify",
            "makes everything kinda more intense",
            r"^\[.*?\]$",
            catinator::hooks::intensify
        ),
        command(
            "pet",
            "Pet the cat, cats generally like pets.",
            catinator::hooks::pet::pet
        ),
        command(
            "about",
            "Prints some info about this kitty cat",
            catinator::hooks::about
        ),
    ];
}
