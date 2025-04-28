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
        r"\b(?:muito bom|super legal|extremamente bom|bem legal|tão bom|demais bom|bastante bom|altamente legal|totalmente bom|incrivelmente bom|absurdamente bom|fora de série bom|decepcionante|péssimo|ruim|horrível|desastroso|genérico|estranho|bizarro|incrível|bom|legal|divertido|fácil|maravilhoso|ótimo|interessante|fantástico|excepcional|sensacional|show|foda|top|caramba|absurdo|triste|lamentável|impressionante|engraçado|inovador|infantil|chato|tenso|emocionante|mágico|maravilhosa|excepcional|brilhante|inspirador|positivo|negativo|surpreendente|confortável|entediado|perfeito|útil|delicioso|adorável|cativante|animado|gostoso|interessante|desagradável|forte|suave|satisfatório|medíocre|impulsivo|agitado|desapontado|maravilhosos|excelente|magnífico|magníficos|magnífica|magníficas|maravilhoso|maravilhosa|maravilhosos|maravilhosas|espetacular|espetaculares|perfeito|perfeitos|perfeita|perfeitas|brilhante|inspirador|fantástico|incrível|sensacional|excepcional)\b|\b(?:extremamente|muito|super|bem)\s?(?:bom|boa|boas|bons|maravilhoso|maravilhosa|maravilhosos|maravilhosas|excelente|magnífico|magníficos|magnífica|magníficas|espetacular|espetaculares|perfeito|perfeitos|perfeita|perfeitas|brilhante|inspirador|fantástico|incrível|sensacional|excepcional|ótimo|ótimos|ótima|ótimas|agradável|agradáveis|positivo|divertido|ruim|péssimo|horrível|medíocre|decepcionante|desagradável|genérico|estranho|desapontado)\b"
    ).unwrap()
}

fn peso_palavra(palavra: &str) -> i32 {
    match palavra {
        "excelente" | "magnífico" | "magníficos" | "magnífica" | "magníficas" |
        "maravilhoso" | "maravilhosa" | "maravilhosos" | "maravilhosas" |
        "espetacular" | "espetaculares" |
        "perfeito" | "perfeitos" | "perfeita" | "perfeitas" |
        "brilhante" | "inspirador" | "fantástico" |
        "incrível" | "sensacional" | "excepcional" => 5,

        "ótimo" | "ótimos" | "ótima" | "ótimas" |
        "bom" | "bons" | "boa" | "boas" |
        "agradável" | "agradáveis" | "positivo" | "divertido" => 3,

        "genérico" | "estranho" | "entediado" |
        "desapontado" => -2,

        "ruim" | "ruins" |
        "terrível" | "terríveis" |
        "péssimo" | "péssimos" | "péssima" | "péssimas" |
        "desagradável" | "desagradáveis" | "bizarro" |
        "medíocre" | "decepcionante" => -5,

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
