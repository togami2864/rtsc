use rtsc_parser::Lexer;

fn main() {
    let code = "class C extends null {
        constructor() {
            super();
            return Object.create(null);
        }
    }

    class D extends null {
        constructor() {
            return Object.create(null);
        }
    }";
    let l = Lexer::new(code);
    let (tokens, errors) = l.lex();

    if errors.is_empty() {
        println!("Tokens: {:?}", tokens);
    } else {
        for e in errors {
            let e = e.with_source_code(code);
            println!("Error: {:?}", e);
        }
    }
}
