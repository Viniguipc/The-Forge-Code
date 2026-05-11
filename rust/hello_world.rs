fn main() {
    println!("Hello, world!"); // Função de escrita no terminal

    let x = 5; // let declara uma variável imutável por padrão
    println!("O valor de x é: {}", x);

    let mut y = 5; // mut declara uma variável mutável
    println!("O valor de y é: {}", y);
    y = 10; // Agora podemos mudar o valor de y
    println!("O novo valor de y é: {}", y);

    //Rust consegue inferir o tipo de variável, mas para especificar o tipo usar : tipo
    let z: i32 = 7; // inteiro com sinal de 32 bits
    let w: u32 = 8; // inteiro sem sinal de 32 bits
    let v: f32 = 9.1; // float 32 bits
    let u: f64 = 10.1; // float 64 bits (padrão)
    println!("O valor de z é: {}", z);
    println!("O valor de w é: {}", w);
    println!("O valor de v é: {}", v);
    println!("O valor de u é: {}", u);

    // Owership
    let s1 = String::from("hello");
    let s2 = s1; // s2 recebe o valor de s1, e s1 perde o valor
    println!("{}", s2); // Funciona pois s2 tem o valor, se fosse s1 daria erro

    let s3 = String::from("hello");
    let s4 = s3.clone(); // s4 recebe o valor de s3, e s3 mantém o valor
    println!("{}", s4);
    println!("{}", s3); // Funciona pois s3 tem o valor

    // Estruturas de controle (if, loop, ...)
    let numero = 10;

    //condicional (if)
    if numero > 5 {
        println!("O número é maior que 5");
    } else {
        println!("O número é menor ou igual a 5");
    }

    //loop for
    for i in 1..=10 {
        // 1..=10 é inclusivo, ou seja, de 1 até 10, sem o = seria até 9
        println!("{}", i);
    }
}
