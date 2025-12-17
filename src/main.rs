#[tokio::main]
async fn main() {
    use catinator::catinator;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    rustls::crypto::CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider())
        .unwrap();

    let mut bot = catinator::Bot::new().await.unwrap();

    let mut sed = catinator::hooks::sed::Sed::new();
    let wolfram_alpha = catinator::hooks::wolfram_alpha::WolframAlpha::new(&bot)
        .expect("failed to initialize WolframAlpha command");

    catinator![
        // hook(
        //     "sasl",
        //     "Handle Authentication.",
        //     AUTHENTICATE,
        //     catinator::hooks::sasl
        // ),
        hook(
            "sed_log",
            "Log messages for use with sed replace, max 10k lines.",
            PRIVMSG,
            sed.log
        ),
        matcher(
            "nitter",
            "replace twitter urls with a nitter instance (xcancel.com)",
            r"https:\/\/(twitter|x)\.com\/",
            catinator::hooks::nitter
        ),
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
            catinator::hooks::pet
        ),
        command(
            "about",
            "Prints some info about this kitty cat",
            catinator::hooks::about
        ),
        async command(
            "wa",
            "Returns Wolfram Alpha results for a query",
            wolfram_alpha.wa
        ),
    ];
}
