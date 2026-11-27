use kuhkey_core::Entry;

// TODO: replace with the real vault once the ext can store one
pub fn entries() -> Vec<Entry> {
    let e =
        |name: &str, username: &str, password: &str| Entry::new(name, password, username, "", "");

    vec![
        e("GitHub", "yoshi-dev", "muh-1234"),
        e("Email", "yoshi.kuh@example.com", "grass-is-green"),
        e("Bank", "DE-4471-2093", "milk&cookies!"),
        e("Netflix", "moovie.night@example.com", "Moo!Flix#2026"),
        e("Spotify", "cowbell_beats", "cowbell-4ever"),
        e("Amazon", "hay.shopper@example.de", "Hay$Bale_77"),
        e("Discord", "UdderChaos#0420", "udder.chaos.9"),
        e("Steam", "pasture_gamer", "pasture-gamer-42"),
        e("Reddit", "u/holy_cow_99", "r/cows<3moo"),
        e("Google", "grazing.field@gmail.example", "Gr4zing!Field"),
        e(
            "University Portal",
            "s1234567@uni.example.de",
            "semester_Kuh_WS26",
        ),
        e(
            "Very Long Name For Testing",
            "a.very.long.username.for.layout.testing@example.com",
            "xK9#mP2$vL7@qR4!nB8&wT5^zF3*hJ6",
        ),
    ]
}
