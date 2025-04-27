use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::io::Write;
use strsim::jaro_winkler;

fn carregar_adjetivos<P: AsRef<Path>>(caminho: P, negativo: bool) -> HashMap<String, i32> {
    let mut mapa = HashMap::new();
    let conteudo = fs::read_to_string(caminho).unwrap_or_default();

    for linha in conteudo.lines() {
        let partes: Vec<&str> = linha.split('-').map(|s| s.trim()).collect();
        if partes.len() == 2 {
            if let Ok(mut valor) = partes[1].parse::<i32>() {
                if negativo {
                    valor *= -1;
                }
                mapa.insert(partes[0].to_lowercase(), valor);
            }
        }
    }

    mapa
}

fn carregar_todos_adjetivos() -> HashMap<String, i32> {
    let mut mapa = HashMap::new();

    let arquivos = vec![
        ("adjetivos_filme_positivos.txt", false),
        ("adjetivos_filme_positivos2.txt", false),
        ("adjetivos_filme_negativos.txt", true),
        ("adjetivos_filme_negativos2.txt", true),
    ];

    for (arquivo, negativo) in arquivos {
        let novo = carregar_adjetivos(arquivo, negativo);
        mapa.extend(novo);
    }

    mapa
}

fn pontuar_comentario(comentario: &str, notas: &HashMap<String, i32>) -> i32 {
    let comentario = comentario.to_lowercase();
    let mut nota_total = 0;

    let palavras: Vec<&str> = comentario.split_whitespace().collect();

    for (expressao, valor) in notas {
        if comentario.contains(expressao) {
            nota_total += *valor;
        } else {
            for palavra in &palavras {
                if jaro_winkler(palavra, expressao) > 0.95 {
                    nota_total += *valor;
                    break;
                }
            }
        }
    }

    nota_total
}

fn classificar_comentario(nota: i32) -> &'static str {
    match nota {
        i32::MIN..=-8 => "Muito negativo",
        -7..=-3 => "Negativo",
        -2..=2 => "Neutro",
        3..=7 => "Positivo",
        8..=i32::MAX => "Muito positivo",
    }
}

fn processar_comentarios() {
    let comentarios = fs::read_to_string("comentarios.txt").unwrap_or_default();
    let notas = carregar_todos_adjetivos();
    
    fs::create_dir_all("resultados").expect("Erro ao criar a pasta de resultados");
    
    let mut arquivo = File::create("resultados/comentarios_avaliados.txt")
        .expect("Erro ao criar arquivo de resultados");

    for (i, comentario) in comentarios.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let comentario = comentario.trim();
        let nota = pontuar_comentario(comentario, &notas);
        let classificacao = classificar_comentario(nota);

        println!("{}º Comentário: \"{}\"", i + 1, comentario);
        println!("Nota: {}\nClassificação: {}\n", nota, classificacao);

        let linha = format!("{} | Nota: {} | Classificação: {}\n", comentario, nota, classificacao);
        arquivo.write_all(linha.as_bytes()).expect("Erro ao escrever no arquivo");
    }
}

fn main() {
    processar_comentarios();
}
