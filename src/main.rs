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
        async hook(
            "url_preview",
            "Send preview of website",
            PRIVMSG,
            catinator::hooks::url::url_preview
        )
    ];
}
