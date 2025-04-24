use std::collections::HashMap;
use std::fs;
use regex::Regex;
use strsim::jaro_winkler;

/// Lê um arquivo de notas e converte para um HashMap.
/// Se `inverter` for true, os valores serão multiplicados por -1.
fn carregar_notas(path: &str, inverter: bool) -> HashMap<String, i32> {
    let mut mapa = HashMap::new();
    let regex = Regex::new(r"^\s*(.+?)\s*-\s*(-?\d+)\s*$").unwrap();

    if let Ok(conteudo) = fs::read_to_string(path) {
        for linha in conteudo.lines() {
            if let Some(captura) = regex.captures(linha) {
                let palavra = captura[1].trim().to_lowercase();
                if let Ok(mut valor) = captura[2].parse::<i32>() {
                    if inverter {
                        valor *= -1;
                    }
                    mapa.insert(palavra, valor);
                }
            }
        }
    }

    mapa
}

/// Carrega todos os arquivos de notas em um único HashMap
fn carregar_todas_as_notas() -> HashMap<String, i32> {
    let mut total = HashMap::new();

    let positivos = [
        "adjetivos_filme_positivos.txt",
        "adjetivos_filme_positivos2.txt",
    ];

    let negativos = [
        "adjetivos_filme_negativos.txt",
        "adjetivos_filme_negativos2.txt",
    ];

    for arquivo in positivos.iter() {
        total.extend(carregar_notas(arquivo, false));
    }

    for arquivo in negativos.iter() {
        total.extend(carregar_notas(arquivo, true));
    }

    total
}

/// Lê os comentários do arquivo (separados por linhas em branco)
fn carregar_comentarios(path: &str) -> Vec<String> {
    let mut comentarios = Vec::new();

    if let Ok(conteudo) = fs::read_to_string(path) {
        for bloco in conteudo.split("\n\n") {
            let bloco = bloco.trim();
            if !bloco.is_empty() {
                comentarios.push(bloco.to_string());
            }
        }
    }

    comentarios
}

/// Calcula a nota total de um comentário com base nas palavras
fn pontuar_comentario(comentario: &str, notas: &HashMap<String, i32>) -> i32 {
    let comentario = comentario.to_lowercase();
    let mut nota_total = 0;
    let mut ja_usados = vec![false; comentario.len()];

    for (expressao, valor) in notas {
        if let Some(mut i) = comentario.find(expressao) {
            while i < comentario.len() {
                let fim = i + expressao.len();
                if ja_usados[i..fim.min(ja_usados.len())].iter().all(|&b| !b) {
                    nota_total += *valor;
                    for j in i..fim.min(ja_usados.len()) {
                        ja_usados[j] = true;
                    }
                }
                if let Some(prox) = comentario[i + 1..].find(expressao) {
                    i = i + 1 + prox;
                } else {
                    break;
                }
            }
        } else {
            for palavra in comentario.split_whitespace() {
                let score = jaro_winkler(palavra, expressao);
                if score > 0.95 {
                    nota_total += *valor;
                    break;
                }
            }
        }
    }

    nota_total
}

fn main() {
    let mapa_de_notas = carregar_todas_as_notas();
    let comentarios = carregar_comentarios("comentarios.txt");

    for comentario in comentarios {
        let nota = pontuar_comentario(&comentario, &mapa_de_notas);
        println!("Comentário: \"{}\"\nNota total: {}\n", comentario, nota);
    }
}
