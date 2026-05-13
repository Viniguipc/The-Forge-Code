//Lista Simplesmente Encadeada (LSE)
//Lista Duplamente Encadeada (LDE)
//Arvore Binaria de Busca (ABB)

use std::io::{self, Write};

struct Lista {}

fn menu() -> u8 {
    let mut op: u8 = 0;
    while op < 1 || op > 4 {
        println!("1 - Inserir elemento");
        println!("2 - Remover elemento");
        println!("3 - Exibir lista");
        println!("4 - Sair");
        print!("Digite sua opção: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Falha ao ler linha");
        
        op = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => 0,
        };
    }

    op
}

fn main() {
    let mut op: u8 = 0;
    while op != 4 {
        op = menu();
        match op {
            1 => println!("Inserir elemento"),
            2 => println!("Remover elemento"),
            3 => println!("Exibir lista"),
            4 => println!("Sair"),
            _ => println!("Opção inválida"),
        }
    }
}
