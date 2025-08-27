use crate::modules::tokenizer::Token;

#[derive(Debug, Clone)]
pub enum ASTValue {
    Str(String),
    Int(i32),
}

#[derive(Debug, Clone)]
pub enum AST {
    Import(String),
    VarDecl(String, String, ASTValue), // ty, name, value
    VarRef(String),
    Literal(ASTValue),
    NewLine,
    Call {
        object: String,
        method: String,
        args: Vec<AST>,
    },
}

pub fn parseTokens(tokens: &[Token]) -> Vec<AST> {
    let mut ast = Vec::with_capacity(tokens.len() / 3);
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Let => {
                if let (
                    Some(Token::Ident(ty)),
                    Some(Token::Ident(name)),
                    Some(Token::Eq),
                    Some(val_token),
                ) = (
                    tokens.get(i + 1),
                    tokens.get(i + 2),
                    tokens.get(i + 3),
                    tokens.get(i + 4),
                ) {
                    match val_token {
                        Token::String(s) => {
                            ast.push(AST::VarDecl(
                                ty.clone(),
                                name.clone(),
                                ASTValue::Str(s.clone()),
                            ));
                        }
                        Token::Integer(n) => {
                            ast.push(AST::VarDecl(ty.clone(), name.clone(), ASTValue::Int(*n)));
                        }
                        _ => panic!("Expected string or integer literal"),
                    }
                    i += 5;
                } else {
                    panic!("Invalid variable declaration");
                }
            }

            Token::Semicolon => {
                ast.push(AST::NewLine);
                i += 1;
            }

            Token::Use => {
                if let Some(Token::Ident(package)) = tokens.get(i + 1) {
                    ast.push(AST::Import(package.clone()));
                    i += 2;
                } else {
                    panic!("Invalid import!");
                }
            }

            Token::Ident(obj) if tokens.get(i + 1) == Some(&Token::Dot) => {
                if let Some(Token::Ident(method)) = tokens.get(i + 2) {
                    if tokens.get(i + 3) == Some(&Token::LParen) {
                        let mut args = Vec::new();
                        let mut j = i + 4;

                        while let Some(tok) = tokens.get(j) {
                            match tok {
                                Token::RParen => break,
                                Token::String(s) => args.push(AST::Literal(ASTValue::Str(s.clone()))),
                                Token::Integer(n) => args.push(AST::Literal(ASTValue::Int(*n))),
                                Token::Ident(name) => args.push(AST::VarRef(name.clone())),
                                Token::Comma => {} // skip
                                _ => panic!("Unexpected token in args: {:?}", tok),
                            }
                            j += 1;
                        }

                        if tokens.get(j) != Some(&Token::RParen) {
                            panic!("Expected closing paren");
                        }
                        ast.push(AST::Call {
                            object: obj.clone(),
                            method: method.clone(),
                            args,
                        });
                        i = j + 1;
                    } else {
                        panic!("Expected '(' after method {}", method);
                    }
                } else {
                    panic!("Expected method after '{}.'", obj);
                }
            }

            other => panic!("Unexpected token: {:?}", other),
        }
    }

    ast
}
