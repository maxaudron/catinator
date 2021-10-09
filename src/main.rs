#[tokio::main]
async fn main() {
    use catinator::catinator;

    tracing_subscriber::fmt()
        .compact()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

    let mut sed = catinator::hooks::sed::Sed::new();

    catinator![
        hook(
            "sasl",
            "Handle Authentication.",
            AUTHENTICATE,
            catinator::hooks::sasl
        ),
        hook(
            "url_preview",
            "Send preview of website",
            PRIVMSG,
            catinator::hooks::url::url_preview
        )
    ];
}
