#[tokio::main]
async fn main() {
    use catinator::catinator;

    tracing_subscriber::fmt()
        .compact()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

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
            catinator::hooks::sed::log
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
            catinator::hooks::sed::replace
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
    ];
}
