use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use super::types::ChatAnalysis;

// Podstawowe stopwords dla języka polskiego
static STOPWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["i","w","na","z","do","że","to","jest","się","ten","tak","nie","ale"]
        .iter().cloned().collect()
});

// Synonimy dla rozszerzenia słów kluczowych
static SYNONYMS: Lazy<HashMap<&'static str, HashSet<&'static str>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("piękny", ["ładny", "uroczy", "cudowny", "śliczny"]
        .iter().cloned().collect());
    m.insert("noc", ["wieczór", "ciemność", "zmrok"]
        .iter().cloned().collect());
    m.insert("zmęczony", ["senny", "wyczerpany", "znużony"]
        .iter().cloned().collect());
    m.insert("pomóc", ["wsparcie", "asysta", "pomoc"]
        .iter().cloned().collect());
    m.insert("rozmowa", ["dialog", "konwersacja", "pogawędka"]
        .iter().cloned().collect());
    m.insert("informacja", ["dane", "fakty", "wiadomości"]
        .iter().cloned().collect());
    m.insert("wiedza", ["informacje", "mądrość", "znajomość"]
        .iter().cloned().collect());
    m
});

// Słowniki emocji
static EMOTIONS: Lazy<HashMap<&'static str, HashSet<&'static str>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("radość", ["szczęśliwy","radość","uśmiech","zadowolony","entuzjazm","super"]
        .iter().cloned().collect());
    m.insert("smutek", ["smutny","przygnębiony","żal","płacz","depresja","melancholia"]
        .iter().cloned().collect());
    m.insert("złość", ["zły","wkurzony","wściekły","gniew","irytacja","nienawiść"]
        .iter().cloned().collect());
    m.insert("neutralny", ["ok","dobrze","okej","niema","cisza"]
        .iter().cloned().collect());
    m
});
// Potoczne i formalne
static INFORMAL: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["elo","no","w sumie","wiesz","lol","xd","spoko","masakra","kurde","kurna"]
        .iter().cloned().collect()
});
static FORMAL: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["szanowni","państwo","uprzejmie","z poważaniem","proszę","dziękuję","z wyrazami szacunku"]
        .iter().cloned().collect()
});
// Tematy
static TOPICS: Lazy<HashMap<&'static str, HashSet<&'static str>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("edukacja", ["szkoła","uczeń","nauka","lekcja","kurs","egzamin","wykład"]
        .iter().cloned().collect());
    m.insert("technologia", ["komputer","internet","programowanie","ai","sztuczna inteligencja","aplikacja","algorytm"]
        .iter().cloned().collect());
    m.insert("emocje", ["radość","smutek","złość","strach","miłość","zazdrość","emocje"]
        .iter().cloned().collect());
    m.insert("egzystencjalne", ["życie","sens","istnienie","filozofia","wszechświat","przeznaczenie","pytanie"]
        .iter().cloned().collect());
    m.insert("relacje", ["przyjaźń","rodzina","związek","rozwód","samotność","zdrada","miłość"]
        .iter().cloned().collect());
    m
});

/// Prosta tokenizacja: rozdziela po niealfanumerycznych i usuwa puste
fn tokenize(msg: &str) -> Vec<String> {
    msg.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

pub fn analyze_message(msg: &str) -> ChatAnalysis {
    let mut analysis = ChatAnalysis::new();
    let tokens = tokenize(msg);

    // 1. Keywords: frekwencja minus stopwords
    let mut freq = HashMap::new();
    for t in &tokens {
        if !STOPWORDS.contains(t.as_str()) {
            *freq.entry(t).or_insert(0usize) += 1;
        }
    }
    // wybór top 3 słów kluczowych
    let mut keywords: Vec<String> = freq.iter()
        .filter(|&(_, &c)| c > 1)
        .map(|(&t, _)| t.clone())
        .collect();

    // Rozszerz słowa kluczowe o synonimy
    let mut extended_keywords = keywords.clone();
    for key in &keywords {
        if let Some(syns) = SYNONYMS.get(key.as_str()) {
            for syn in syns {
                if !extended_keywords.contains(&syn.to_string()) {
                    extended_keywords.push(syn.to_string());
                }
            }
        }
    }
    if extended_keywords.len() < 3 {
        let mut sorted: Vec<_> = freq.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (&t, _) in sorted.iter().take(3) {
            let t = t.clone();
            if !extended_keywords.contains(&t) {
                extended_keywords.push(t);
            }
        }
    }
    keywords = extended_keywords;
    analysis.set_keywords(keywords);

    // 2. Sentence type
    let st = if msg.trim().ends_with('?') {
        if msg.chars().count() > 20 {"pytanie"} else {"krótkie pytanie"}
    } else if msg.trim().ends_with('!') {
        "wykrzyknik"
    } else {
        "stwierdzenie"
    };
    analysis.sentence_type = st.to_string();
    analysis.set_weight("sentence_type", 1.0);

    // 3. Emocje
    let mut emo_scores = HashMap::new();
    for (emo, dict) in EMOTIONS.iter() {
        emo_scores.insert(emo, tokens.iter().filter(|t| dict.contains(t.as_str())).count());
    }
    let (mood, score) = emo_scores.iter()
        .max_by_key(|&(_, &c)| c).map(|(&e,&c)| (e, c)).unwrap_or((&"neutralny",0));
    analysis.mood = mood.to_string();
    analysis.set_weight("mood", score as f32);

    // 4. Cel
    let goal = if tokens.iter().any(|t| t == "chcę" || t == "potrzebuję") {
        "wyrażenie potrzeby"
    } else if tokens.iter().any(|t| t == "jak" || t == "czy" || t == "dlaczego") {
        "pozyskanie informacji"
    } else if tokens.iter().any(|t| t == "dziękuję" || t == "proszę") {
        "grzeczność"
    } else {
        "rozmowa ogólna"
    };
    analysis.goal = goal.to_string();
    analysis.set_weight("goal", if goal == "rozmowa ogólna" {0.5} else {1.0});

    // 5. Intencja
    let intent = if tokens.iter().any(|t| t == "pomóż" || t == "proszę") {
        "prośba o pomoc"
    } else if tokens.iter().any(|t| t == "dziękuję" || t == "przepraszam") {
        "grzeczność"
    } else {
        "komunikacja"
    };
    analysis.intent = intent.to_string();
    analysis.set_weight("intent", if intent == "prośba o pomoc" {1.0} else {0.5});

    // 6. Styl formalny vs potoczny
    let informal = tokens.iter().filter(|t| INFORMAL.contains(t.as_str())).count() as f32;
    let formal = tokens.iter().filter(|t| FORMAL.contains(t.as_str())).count() as f32;
    let style = if formal>informal {"formalny"} else if informal>0.0 {"potoczny"} else {"neutralny"};
    analysis.representation = style.to_string();
    analysis.set_weight("style_formal", formal);
    analysis.set_weight("style_informal", informal);

    // 7. Tematyka
    let mut topic_scores = HashMap::new();
    for (tpc, dict) in TOPICS.iter() {
        topic_scores.insert(tpc, tokens.iter().filter(|t| dict.contains(t.as_str())).count());
    }
    let (topic, tscore) = topic_scores.iter().max_by_key(|&(_, &c)| c)
        .map(|(&t,&c)|(t,c)).unwrap_or((&"ogólny",0));
    analysis.set_weight(&format!("topic_{}", topic), tscore as f32);

    // 8. Streszczenie
    let max = 100.min(msg.len());
    let mut sum = msg[..max].to_string();
    if sum.len()==max && msg.len()>max {
        if let Some(pos)=sum.rfind(' ') { sum.truncate(pos); }
        sum.push('…');
    }
    analysis.content_summary = sum;
    analysis.set_weight("summary_length", analysis.content_summary.len() as f32);

    // 9. Wizja
    let vis = match goal {
        "pozyskanie informacji" => "uzyskanie wiedzy",
        "wyrażenie potrzeby" => "zaspokojenie potrzeby",
        _ => "kontynuacja dialogu",
    };
    analysis.vision = vis.to_string();
    analysis.set_weight("vision", 1.0);

    analysis
}
