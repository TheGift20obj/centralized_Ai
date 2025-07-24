// src/response.rs
use super::types::ChatAnalysis;
use ic_cdk::api::time;

/// Bardziej rozbudowany moduł generowania odpowiedzi AI,
/// z szerszą bazą szablonów i lepszym wykorzystaniem keywords

// Templates bazujące na emocjach (każda lista 5 wariantów)
static EMOTION_TEMPLATES: &[(&str, &[&str])] = &[
    ("radość", &[
        "To brzmi fantastycznie!",
        "Super usłyszeć takie informacje!",
        "Cieszę się razem z Tobą!",
        "Wygląda na to, że jesteś bardzo zadowolony!",
        "Twoja radość jest zaraźliwa!",
        "To naprawdę budujące usłyszeć takie słowa!",
    ]),
    ("smutek", &[
        "Przykro mi to słyszeć.",
        "Rozumiem, że możesz czuć się przygnębiony.",
        "Jest mi przykro z tego powodu.",
        "Wspieram Cię w tej trudnej chwili.",
        "Mam nadzieję, że wkrótce poczujesz się lepiej.",
        "Twoje uczucia są dla mnie ważne.",
    ]),
    ("złość", &[
        "Widzę, że jesteś zdenerwowany.",
        "Rozumiem Twoją frustrację.",
        "To musi być dla Ciebie irytujące.",
        "Przykro mi, że odczuwasz złość.",
        "Postaram się pomóc rozładować tę sytuację.",
        "Twoje emocje są zrozumiałe.",
    ]),
    ("neutralny", &[
        "Dziękuję za informację.",
        "Rozumiem.",
        "Dziękuję za podzielenie się.",
        "Zrozumiałem przekaz.",
        "Doceniam Twoją wypowiedź.",
        "Twoja wypowiedź jest dla mnie ważna.",
    ]),
];

// Style templates (5 wariantów każdy)
static STYLE_TEMPLATES: &[(&str, &[&str])] = &[
    ("formalny", &[
        "Jeżeli potrzebujesz dalszych wyjaśnień, proszę daj znać.",
        "Proszę o informację, jeśli coś wymaga doprecyzowania.",
        "Służę pomocą w razie pytań.",
        "Zapraszam do zadawania kolejnych pytań.",
        "Jestem do dyspozycji w razie potrzeby.",
        "Z przyjemnością odpowiem na kolejne pytania.",
    ]),
    ("potoczny", &[
        "Daj znać, jeśli coś Cię jeszcze interesuje!",
        "Pytaj śmiało, jestem tu dla Ciebie.",
        "Coś jeszcze poruszyć?",
        "Możesz pytać o wszystko.",
        "Jestem gotów na kolejne pytania!",
        "Śmiało, pytaj dalej!",
    ]),
    ("neutralny", &[
        "Czy jest coś jeszcze, o czym chciałbyś porozmawiać?",
        "Jak mogę Ci jeszcze pomóc?",
        "Masz jeszcze jakieś pytania?",
        "Chętnie odpowiem na kolejne pytania.",
        "Czy potrzebujesz dodatkowych informacji?",
        "Jestem tu, by pomóc w każdej sprawie.",
    ]),
];

/// Tworzy bardziej naturalną i zróżnicowaną frazę z słów kluczowych
fn keywords_phrase(keys: &[String]) -> Option<String> {
    if keys.is_empty() {
        None
    } else {
        let list = keys.join(", ");
        // More natural phrasing with variation
        let phrase = match keys.len() {
            1 => format!("Zauważyłem, że wspomniałeś o słowie: {}.", list),
            2 => format!("Wspomniałeś o słowach: {} i {}.", keys[0], keys[1]),
            _ => {
                let all_but_last = &keys[..keys.len()-1];
                let last = &keys[keys.len()-1];
                format!("Wspomniałeś o słowach: {}, oraz {}.", all_but_last.join(", "), last)
            }
        };
        Some(phrase)
    }
}

/// Podstawowe frazy intencji
fn intent_phrase(a: &ChatAnalysis) -> &'static str {
    match a.intent.as_str() {
        "prośba o pomoc" => "Chętnie pomogę – daj znać, w czym mogę się przydać.",
        "komunikacja" => "Jestem tu, by prowadzić rozmowę.",
        _ => "Czy mogę pomóc w inny sposób?",
    }
}

/// Słownik wybranych słów kluczowych z wyjaśnieniami lub powiązanymi frazami
static KEYWORD_EXPLANATIONS: &[(&str, &str)] = &[
    ("pogoda", "Pogoda to stan atmosfery, który wpływa na nasze samopoczucie."),
    ("pies", "Pies to wierny towarzysz człowieka, często nazywany najlepszym przyjacielem."),
    ("kolor", "Kolor to wrażenie wzrokowe wywołane przez światło odbite od powierzchni."),
    ("noc", "Noc to czas, gdy słońce jest poniżej horyzontu, a świat jest ciemny."),
    ("zmęczony", "Zmęczenie to uczucie braku energii i potrzeby odpoczynku."),
    ("rozmowa", "Rozmowa to wymiana myśli i informacji między ludźmi."),
    ("informacja", "Informacja to dane, które przekazują wiedzę."),
    ("wiedza", "Wiedza to zbiór informacji i doświadczeń."),
    ("pies", "Pies jest zwierzęciem domowym, często towarzyszem ludzi."),
    ("uczucia", "Uczucia to emocje, które odczuwamy w różnych sytuacjach."),
    ("wczoraj", "Wczoraj to dzień poprzedzający dzisiaj."),
];

/// Główna funkcja generowania odpowiedzi AI (skrócona i wzbogacona wersja)
pub fn generate_response(a: &ChatAnalysis) -> String {
    let now = time();
    // Wybór losowy na podstawie timestampu
    let emo_phrases = EMOTION_TEMPLATES.iter().find(|&&(e, _)| e == a.mood)
        .map(|&(_, ph)| ph).unwrap_or(&EMOTION_TEMPLATES[3].1);
    let emo = emo_phrases[(now % emo_phrases.len() as u64) as usize];

    let style_phrases = STYLE_TEMPLATES.iter().find(|&&(s, _)| s == a.representation)
        .map(|&(_, ph)| ph).unwrap_or(&STYLE_TEMPLATES[2].1);
    let style = style_phrases[((now/2) % style_phrases.len() as u64) as usize];

    let intent = intent_phrase(a);

    // Buduj odpowiedź z wyjaśnieniami słów kluczowych, jeśli dostępne
    let mut keyword_sentences = Vec::new();
    for kw in a.keywords.iter().take(3) {
        if let Some(&(_, explanation)) = KEYWORD_EXPLANATIONS.iter().find(|&&(word, _)| word == kw) {
            keyword_sentences.push(format!("{} {}", " ", explanation));
        }
    }

    // Jeśli brak wyjaśnień, użyj krótkiego wplecenia słów kluczowych
    /*if keyword_sentences.is_empty() && !a.keywords.is_empty() {
        keyword_sentences.push(format!("Wspomniałeś o: {}.", a.keywords.iter().take(2).cloned().collect::<Vec<_>>().join(", ")));
    }*/

    // Buduj krótką odpowiedź z kluczowymi elementami
    let mut resp = vec![
        emo.to_string(),
    ];
    resp.extend(keyword_sentences);
    //resp.push(format!("Cel: {}.", a.goal));
    //resp.push(format!("Intencja: {}.", a.intent));
    //resp.push(intent.to_string());
    //resp.push(style.to_string());

    // Dodaj krótkie zdanie w zależności od celu
    if a.goal == "pozyskanie informacji" {
        resp.push("Chętnie pomogę.".to_string());
    } else if a.goal == "wyrażenie potrzeby" {
        resp.push("Postaram się pomóc.".to_string());
    } else if a.goal == "grzeczność" {
        resp.push("Dziękuję.".to_string());
    } else {
        resp.push("Jestem tu, by rozmawiać.".to_string());
    }

    // Filtruj puste
    resp.retain(|s| !s.is_empty());
    resp.join(" ")
}

/// Generuje bazę wariacji (~500) opartych na analizie
pub fn generate_variations(a: &ChatAnalysis) -> Vec<String> {
    let mut vs = Vec::new();
    // Kombinacje emocji, stylu i keyword
    for &(emo, phs) in EMOTION_TEMPLATES.iter() {
        for &ep in phs.iter() {
            for &(st, phs2) in STYLE_TEMPLATES.iter() {
                for &sp in phs2.iter() {
                    let kw = keywords_phrase(&a.keywords).unwrap_or_default();
                    let v = format!(
                        "{} {} Z tego: {}. {} {}. {}",
                        ep, kw, a.goal, intent_phrase(a), sp, a.vision
                    );
                    vs.push(v);
                    if vs.len() >= 500 { return vs; }
                }
            }
        }
    }
    // Uzupełnij cyklicznie
    let mut i = 0;
    while vs.len() < 500 {
        vs.push(vs[i % vs.len()].clone());
        i += 1;
    }
    vs
}
