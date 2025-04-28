use fancy_regex::Regex;
use serde::Serialize;
use strsim::jaro_winkler;
use std::fs::{self};

#[derive(Debug, Serialize)]
struct DetalheAnalise {
    trecho: String,
    nota: i32,
    intensificador: Option<String>,
    ironia_detectada: bool,
}

#[derive(Debug, Serialize)]
struct AnaliseComentario {
    comentario: String,
    nota: i32,
    classificacao: String,
    detalhes: Vec<DetalheAnalise>,
}

fn carregar_padrao() -> Regex {
    Regex::new(
        r"\b(?:muito bom|super legal|extremamente bom|bem legal|tão bom|demais bom|bastante bom|altamente legal|totalmente bom|incrivelmente bom|absurdamente bom|fora de série bom|perfeito|extraordinário|excelente|ótimo|maravilhoso|fantástico|genial|transcendental|primoroso|formidável|excepcional|incrível|inesquecível|magistral|fenomenal|poderoso|sensacional|esplêndido|bonito|emocionante|profundo|expressivo|impactante|delicado|impressionante|grandioso|épico|exuberante|fascinante|cativante|encantador|envolvente|apaixonante|carismático|reflexivo|surpreendente|tocante|instigante|revolucionário|inovador|memorável|emocional|realista|comovente|criativo|dinâmico|cinematográfico|ousado|agradável|divertido|acolhedor|interessante|amigável|equilibrado|engajante|complexo|ambicioso|fofo|engraçado|simpático|doce|autêntico|original|eficaz|poético|relevante|maduro|notável|leve|bom|bons|perturbador|sinistro|grotesco|medonho|anêmico|deplorável|desumano|intolerante|horrível|horrendo|doentio|indigesto|desfigurando|desfigurante|arrogante|metido|tóxico|inexpressivo|mortificado|medíocre|ordinário|péssimo|trágico|ridículo|ultrapassado|careta|anticlimático|pífio|pretensioso|problemático|arrastado|apático|genérico|desconexo|confuso|desinteressante|pesado|superestimado|obsoleto|datado|melancólico|enjoativo|estranho|inútil|pobre|insensível|desinformação|tendencioso|malicioso|dispensável|disperso|fragmentado|inconsistente|inconsistência|cansativo|frouxo|exagerado|simplista|vacilante|irresponsável|desconfortável|turbulento|incapaz|inepto|vazio|preguiçoso|repetitivo|desgastante|impróprio|desanimado|desanimador|robotizado|limitado|entediante|tedioso|superficial|inferior|desgastado|apagado|desajeitado|rígido|falso|inverossímil|cru|secundário|artificial|amador|despreparado|malfeito|precário|desprovido|clichê|afobado|desnecessário|imperceptível|penoso|óbvio|batido|piegas|triste|decepcionante|banal|banalizado|exaurido|chato|fraco|previsível|vago|incômodo|áspero|tosco|grosseiro|rudimentar|ralo|raso|infantil|ingênuo|redundante|quebrado|vergonhoso|tosquinho)\b"
    ).unwrap()
}

fn peso_palavra(palavra: &str) -> i32 {    match palavra {
    "perfeito" => 10,

    "extraordinário" => 8,

    "excelente" | "ótimo" | "maravilhoso" | "fantástico" | "genial" | "transcendental" |
    "primoroso" | "formidável" | "excepcional" | "incrível" => 7,

    "inesquecível" | "magistral" | "fenomenal" | "poderoso" | "sensacional" | "esplêndido" => 6,

    "bonito" | "emocionante" | "profundo" | "expressivo" | "impactante" |
    "delicado" | "impressionante" | "grandioso" | "épico" | "exuberante" |
    "fascinante" => 5,

    "cativante" | "encantador" | "envolvente" | "apaixonante" | "carismático" |
    "reflexivo" | "surpreendente" | "tocante" | "instigante" | "revolucionário" |
    "inovador" | "memorável" | "emocional" | "realista" | "comovente" |
    "criativo" | "dinâmico" | "cinematográfico" | "ousado" => 4,

    "agradável" | "divertido" | "acolhedor" | "interessante" |
    "amigável" | "equilibrado" | "engajante" | "complexo" | "ambicioso" => 3,

    "fofo" | "engraçado" | "simpático" | "doce" | "autêntico" | "original" |
    "eficaz" | "poético" | "relevante" | "maduro" | "notável" => 2,

    "leve" | "bom" | "bons" => 1,

    // --- Palavras negativas ---

    "perturbador" => -8,

    "sinistro" | "grotesco" | "medonho" | "anêmico" | "deplorável" | "desumano" | "intolerante" => -7,

    "horrível" | "horrendo" | "doentio" | "indigesto" |
    "desfigurando" | "desfigurante" | "arrogante" | "metido" |
    "tóxico" | "inexpressivo" | "mortificado" | "medíocre" | "ordinário" => -6,

    "péssimo" | "trágico" | "ridículo" | "ultrapassado" | "careta" |
    "anticlimático" | "pífio" | "pretensioso" | "problemático" |
    "arrastado" | "apático" => -5,

    "genérico" | "desconexo" | "confuso" | "desinteressante" |
    "pesado" | "superestimado" | "obsoleto" | "datado" |
    "melancólico" | "enjoativo" | "estranho" | "inútil" | "pobre" |
    "insensível" | "desinformação" | "tendencioso" | "malicioso" |
    "dispensável" | "disperso" | "fragmentado" | "inconsistente" |
    "inconsistência" => -4,

    "cansativo" | "frouxo" | "exagerado" | "simplista" |
    "vacilante" | "irresponsável" | "desconfortável" | "turbulento" |
    "incapaz" | "inepto" | "vazio" | "preguiçoso" | "repetitivo" |
    "desgastante" | "impróprio" | "desanimado" | "desanimador" |
    "robotizado" | "limitado" => -3,

    "entediante" | "tedioso" | "superficial" | "inferior" |
    "desgastado" | "apagado" | "desajeitado" | "rígido" | "falso" |
    "inverossímil" | "cru" | "secundário" | "artificial" | "amador" |
    "despreparado" | "malfeito" | "precário" | "desprovido" |
    "clichê" | "afobado" | "desnecessário" | "imperceptível" |
    "penoso" | "óbvio" | "batido" | "piegas" | "triste" |
    "decepcionante" | "banal" | "banalizado" | "exaurido" => -2,

    "chato" | "fraco" | "previsível" | "vago" | "incômodo" |
    "áspero" | "tosco" | "grosseiro" | "rudimentar" | "ralo" |
    "raso" | "infantil" | "ingênuo" | "redundante" | "quebrado" |
    "vergonhoso" | "tosquinho" => -1,

    _ => 0,
}
}

fn similaridade_palavras(palavra1: &str, palavra2: &str) -> f64 {
    jaro_winkler(palavra1, palavra2)
}

fn aplicar_intensificador(base: i32, intensificador: Option<&str>) -> i32 {
    match intensificador {
        Some("extremamente") => base * 2,
        Some("muito") => base * 2,
        Some("super") => base * 2,
        Some("bem") => (base * 3) / 2,
        _ => base,
    }
}

fn pontuar_comentario(comentario: &str) -> AnaliseComentario {
    let padrao = carregar_padrao();
    let comentario_minusculo = comentario.to_lowercase();
    let mut detalhes = Vec::new();
    let mut nota_total = 0;
    let ironia_detectada = comentario_minusculo.contains("só que não");

    let captures = padrao.captures_iter(&comentario_minusculo);
    for captura in captures {
        if let Ok(captura) = captura {
            if captura.get(0).is_none() {
                continue;
            }
            let trecho = captura.get(0).unwrap().as_str();
            let palavras: Vec<&str> = trecho.split_whitespace().collect();
            let mut local_nota = 0;
            let mut intensificador: Option<String> = None;
            let mut negado = false;

            for (_i, palavra) in palavras.iter().enumerate() {
                match *palavra {
                    "não" => negado = true,
                    "muito" | "extremamente" | "super" | "bem" => {
                        intensificador = Some(palavra.to_string());
                    }
                    _ => {
                        let peso = peso_palavra(palavra);
                        if peso != 0 {
                            let mut peso_final = aplicar_intensificador(peso, intensificador.as_deref());
                            if negado {
                                peso_final *= -1;
                                negado = false;
                            }
                            local_nota += peso_final;
                            
                            for outro_palavra in palavras.iter() {
                                if *outro_palavra != *palavra {
                                    let similaridade = similaridade_palavras(palavra, outro_palavra);
                                    if similaridade > 0.95 {
                                        local_nota += peso_final / 2;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            detalhes.push(DetalheAnalise {
                trecho: trecho.to_string(),
                nota: local_nota,
                intensificador: intensificador.clone(),
                ironia_detectada,
            });

            intensificador = None;
            
            nota_total += local_nota;
        }
    }

    if ironia_detectada {
        nota_total *= -1;
    }

    AnaliseComentario {
        comentario: comentario.to_string(),
        nota: nota_total,
        classificacao: classificar_comentario(nota_total),
        detalhes,
    }
}

fn classificar_comentario(nota: i32) -> String {
    match nota {
        i32::MIN..=-8 => "Muito negativo".to_string(),
        -7..=-3 => "Negativo".to_string(),
        -2..=2 => "Neutro".to_string(),
        3..=7 => "Positivo".to_string(),
        8..=i32::MAX => "Muito positivo".to_string(),
    }
}

fn processar_comentarios() {
    let comentarios = fs::read_to_string("comentarios.txt").unwrap_or_default();

    let mut resultados = Vec::new();

    for comentario in comentarios.lines().filter(|l| !l.trim().is_empty()) {
        let comentario = comentario.trim();
        let analise = pontuar_comentario(comentario);
        resultados.push(analise);
    }

    // Salvar o resultado como JSON
    let json_resultado = serde_json::to_string_pretty(&resultados).expect("Erro ao converter para JSON");
    fs::write("resultados/comentarios_avaliados.json", json_resultado).expect("Erro ao escrever o arquivo JSON");
}

fn main() {
    processar_comentarios();
}
