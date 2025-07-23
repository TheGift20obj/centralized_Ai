
#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[ic_cdk::query]
fn chat(message: String) -> String {
    let msg = message.to_lowercase();
    let words: Vec<&str> = msg.split_whitespace().collect();

    // Intencje z rdzeniami (obsługa odmian)
    let intents = vec![
        ("powitanie", vec!["cześć", "hej", "witaj", "dzień dob", "siem", "elo"]),
        ("pożegnanie", vec!["widzenia", "pa", "nara", "żegnaj", "bye", "trzymaj"]),
        ("pomoc", vec!["pomoc", "help", "wsparc", "problem", "ratunk", "potrzeb"]),
        ("humor", vec!["żart", "śmiesz", "dowcip", "rozśmiesz", "śmiech"]),
        ("tożsamość", vec!["imię", "nazyw", "kim jeste", "czym jeste", "asystent"]),
        ("programowanie", vec!["program", "kod", "rust", "motoko", "algorytm", "debug", "błąd", "framework", "aplikac", "python", "node", "javascript"]),
        ("pogoda", vec!["pogod", "słoń", "deszcz", "zimn", "ciep", "temperatur", "wiatr"]),
        ("podziękowanie", vec!["dzięk", "dziek", "thx", "thanks", "wdzięcz"]),
        ("emocje+", vec!["dobr", "szczęśl", "super", "świetn", "radoś", "zadowol", "ekstra"]),
        ("emocje-", vec!["smutn", "źle", "problem", "nie faj", "przykro", "depres", "zmartw"]),
        ("logika", vec!["logik", "logicz", "rozumow", "wniosk"]),
        ("gry", vec!["gra", "zagra", "gaming", "plansz", "komputer", "rozryw"]),
        ("hobby", vec!["hobby", "zainteres", "pasj", "wolny czas"]),
        ("technologie", vec!["technolog", "nowink", "innowac", "popularn", "trend", "ai", "sztuczn", "blockchain", "react", "vue", "iot", "vr"]),
        ("nauka", vec!["nauk", "uczen", "edukac", "wiedz", "uczyć"]),
        ("motywacja", vec!["motywac", "inspirac", "cytat", "zachęt"]),
        ("ciekawostki", vec!["ciekawost", "fakt", "wiedział", "interes"]),
        ("zdrowie", vec!["zdrow", "fit", "diet", "ćwiczen", "sport"]),
        ("sport", vec!["sport", "piłk", "biegan", "trening", "mecz"]),
        ("muzyka", vec!["muzyk", "piosenk", "utwór", "artyst", "słucha"]),
        ("film", vec!["film", "kino", "serial", "ogląd", "reżyser"]),
        ("podróże", vec!["podróż", "wycieczk", "zwiedz", "wakacj", "miejsce"]),
        ("jedzenie", vec!["jedzen", "potraw", "kuchni", "przepis", "smacz"]),
    ];

    // Konkretne technologie i ich opisy
    let tech_details = vec![
        ("blockchain", "Blockchain to zdecentralizowana baza danych, która umożliwia bezpieczne przechowywanie i przesyłanie informacji. Jest podstawą kryptowalut, takich jak Bitcoin czy Ethereum, oraz wielu nowoczesnych rozwiązań w finansach i logistyce."),
        ("rust", "Rust to nowoczesny język programowania, ceniony za bezpieczeństwo pamięci i wydajność. Używany do tworzenia systemów, aplikacji webowych, gier i narzędzi. Słynie z braku błędów typu 'null pointer' i wyścigów danych."),
        ("motoko", "Motoko to język programowania stworzony specjalnie dla Internet Computer (ICP). Pozwala łatwo pisać smart kontrakty i aplikacje zdecentralizowane."),
        ("ai", "Sztuczna inteligencja (AI) to dziedzina informatyki zajmująca się tworzeniem systemów potrafiących uczyć się, rozumować i podejmować decyzje. Przykłady to rozpoznawanie obrazów, tłumaczenie języków, chatboty."),
        ("react", "React to popularna biblioteka JavaScript do budowy interfejsów użytkownika. Pozwala tworzyć dynamiczne, szybkie aplikacje webowe."),
        ("vue", "Vue.js to lekki framework JavaScript do budowy interfejsów webowych. Jest prosty w nauce i bardzo elastyczny."),
        ("iot", "Internet of Things (IoT) to sieć urządzeń połączonych z internetem, które mogą wymieniać dane i być zdalnie sterowane. Przykłady: inteligentne domy, czujniki, samochody."),
        ("vr", "Virtual Reality (VR) to technologia pozwalająca zanurzyć się w wirtualnym świecie za pomocą specjalnych gogli. Używana w grach, edukacji, medycynie."),
        ("python", "Python to wszechstronny język programowania, bardzo popularny w nauce danych, AI, web development i automatyzacji."),
        ("node", "Node.js to środowisko uruchomieniowe JavaScript pozwalające budować szybkie aplikacje serwerowe."),
        ("javascript", "JavaScript to najpopularniejszy język programowania webowego, używany do tworzenia interaktywnych stron i aplikacji."),
    ];

    let mut detected_intents = Vec::new();
    let mut sentiment = 0;
    let mut found_techs = Vec::new();

    // Detekcja intencji, sentymentu i technologii (obsługa odmian przez starts_with/contains)
    for (intent, keywords) in &intents {
        for word in &words {
            for key in keywords {
                if word.starts_with(key) || word.contains(key) {
                    detected_intents.push(intent.to_string());
                    if *intent == "emocje+" { sentiment += 1; }
                    if *intent == "emocje-" { sentiment -= 1; }
                }
            }
        }
    }
    // Detekcja konkretnych technologii w wiadomości
    for (tech, _) in &tech_details {
        if msg.contains(tech) {
            found_techs.push(tech.to_string());
        }
    }

    detected_intents.sort();
    detected_intents.dedup();
    found_techs.sort();
    found_techs.dedup();

    let mut response_parts = Vec::new();

    // Odpowiedzi dla konkretnych technologii
    for tech in &found_techs {
        for (name, desc) in &tech_details {
            if tech == name {
                response_parts.push(format!("{}: {}", capitalize(name), desc));
            }
        }
    }

    // Dynamiczne budowanie odpowiedzi z wielu intencji
    for intent in &detected_intents {
        match intent.as_str() {
            "powitanie" => response_parts.push("Cześć! Miło Cię widzieć.".to_string()),
            "pożegnanie" => response_parts.push("Do zobaczenia! Jeśli będziesz potrzebować pomocy, wróć do mnie.".to_string()),
            "pomoc" => response_parts.push("Opisz swój problem lub pytanie, a postaram się doradzić albo podpowiedzieć rozwiązanie.".to_string()),
            "humor" => response_parts.push("Oto żart: Dlaczego komputerowi nigdy nie jest zimno? Bo zawsze ma Windows!".to_string()),
            "tożsamość" => response_parts.push("Jestem Twoim wirtualnym asystentem ICP. Pomagam, uczę się i staram się być coraz lepszy!".to_string()),
            "programowanie" => response_parts.push("Programowanie to sztuka rozwiązywania problemów. Zapytaj o kod, algorytmy lub frameworki!".to_string()),
            "pogoda" => response_parts.push("Nie mam dostępu do aktualnej pogody, ale mogę poprawić Twój nastrój rozmową! Jaką pogodę lubisz najbardziej?".to_string()),
            "podziękowanie" => response_parts.push("Nie ma za co! Zawsze służę pomocą. Może chcesz się czegoś dowiedzieć lub pośmiać?".to_string()),
            "logika" => response_parts.push("Logika jest kluczowa w programowaniu i codziennym życiu. Chcesz przykład logicznego zadania?".to_string()),
            "gry" => response_parts.push("Lubisz gry? Polecam planszówki, gry komputerowe i logiczne łamigłówki! W co ostatnio grałeś?".to_string()),
            "hobby" => response_parts.push("Hobby rozwija i daje radość. Jakie są Twoje zainteresowania?".to_string()),
            "technologie" => response_parts.push("Popularne technologie to AI, blockchain, IoT, VR, Rust, Motoko, React, Vue, Python, Node.js, JavaScript. Zapytaj o konkretną technologię, a opowiem więcej!".to_string()),
            "nauka" => response_parts.push("Nauka to klucz do rozwoju. Możesz zapytać mnie o ciekawostki, cytaty lub metody uczenia się.".to_string()),
            "motywacja" => response_parts.push("Oto cytat motywacyjny: 'Nie bój się porażki. To ona uczy Cię najwięcej.' Potrzebujesz więcej inspiracji?".to_string()),
            "ciekawostki" => response_parts.push("Czy wiesz, że pierwsza linia kodu na świecie została napisana przez kobietę – Adę Lovelace? Zapytaj o kolejne ciekawostki!".to_string()),
            "zdrowie" => response_parts.push("Zdrowie jest najważniejsze! Pamiętaj o ruchu, zdrowej diecie i odpoczynku. Chcesz przepis na zdrowe danie?".to_string()),
            "sport" => response_parts.push("Sport to zdrowie i zabawa! Jaki sport lubisz najbardziej?".to_string()),
            "muzyka" => response_parts.push("Muzyka poprawia nastrój i inspiruje. Kogo lubisz słuchać?".to_string()),
            "film" => response_parts.push("Filmy i seriale to świetna rozrywka. Masz ulubiony gatunek lub reżysera?".to_string()),
            "podróże" => response_parts.push("Podróże kształcą i dają niezapomniane wspomnienia. Gdzie chciałbyś pojechać?".to_string()),
            "jedzenie" => response_parts.push("Jedzenie to przyjemność! Masz ulubioną kuchnię lub potrawę?".to_string()),
            _ => {}
        }
    }

    // Sentiment-based enhancement
    if sentiment < 0 {
        response_parts.push("Widzę, że nie jest najlepiej. Jeśli chcesz, opowiem żart, podam cytat lub pomogę w czymś innym.".to_string());
    } else if sentiment > 0 {
        response_parts.push("Cieszę się, że masz dobry nastrój! Może chcesz się czegoś dowiedzieć lub pośmiać?".to_string());
    }

    // Jeśli nie wykryto intencji ani technologii, spróbuj odpowiedzieć dynamicznie
    if response_parts.is_empty() {
        if msg.len() < 10 {
            response_parts.push("Czy możesz napisać coś więcej? Chętnie odpowiem na dłuższe pytania lub opowiem ciekawostkę.".to_string());
        } else if msg.contains("?") {
            response_parts.push("To ciekawe pytanie! Spróbuj je doprecyzować lub zapytaj o programowanie, żart, hobby albo pomoc.".to_string());
        } else {
            response_parts.push(format!(
                "Przeczytałem: '{}'. Staram się odpowiadać najlepiej jak potrafię. Możesz zapytać mnie o gry, hobby, technologie, naukę, motywację, zdrowie, sport, muzykę, filmy, podróże, jedzenie, programowanie, żarty lub poprosić o pomoc.",
                message
            ));
        }
    }

    // Łączenie odpowiedzi w jedną logiczną całość
    let response = response_parts.join(" ");

    response
}

// Pomocnicza funkcja do kapitalizacji nazw technologii
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}