use crate::modules::tokenizer::Token;

#[derive(Debug, Clone)]
pub enum ASTValue {
    Str(String),
    Int(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    VarRef(String),
    FuncCall {
        name: String,
        args: Vec<ASTValue>,
    },
    // Add comparison operations for conditions
    LessThan(Box<ASTValue>, Box<ASTValue>),
    GreaterThan(Box<ASTValue>, Box<ASTValue>),
    Equal(Box<ASTValue>, Box<ASTValue>),
EqualEqual(Box<ASTValue>, Box<ASTValue>),

    NotEqual(Box<ASTValue>, Box<ASTValue>),
    LessEqual(Box<ASTValue>, Box<ASTValue>),
    GreaterEqual(Box<ASTValue>, Box<ASTValue>),
}

#[derive(Debug, Clone)]
pub enum AST {
    Import(String),
    VarDecl(String, String, ASTValue),
    VarRef(String),
    Literal(ASTValue),
    NewLine,
    Call {
        object: String,
        method: String,
        args: Vec<AST>,
    },
    Return(ASTValue),
    FuncDef {
        name: String,
        params: Vec<(String, String)>,
        return_type: Option<String>,
        body: Vec<AST>,
    },
    If {
        condition: ASTValue,
        then_body: Vec<AST>,
        elif_branches: Vec<(ASTValue, Vec<AST>)>, // (condition, body) pairs
        else_body: Option<Vec<AST>>,
    },
}

// Helper function to convert token to type string
fn token_to_type_string(token: &Token) -> Option<String> {
    match token {
        Token::Ident(name) => Some(name.clone()),
        Token::I32Type => Some("i32".to_string()),
        Token::I64Type => Some("i64".to_string()),
        Token::F32Type => Some("f32".to_string()),
        Token::F64Type => Some("f64".to_string()),
        Token::StringType => Some("string".to_string()),
        Token::BoolType => Some("bool".to_string()),
        _ => None,
    }
}

// Helper function to parse let statements (used in both top-level and function body)
fn parse_let_statement(tokens: &[Token], start_index: usize) -> Result<(AST, usize), String> {
    let mut i = start_index + 1; // skip 'let'
    
    if i >= tokens.len() {
        return Err("Unexpected end of tokens after 'let'".to_string());
    }
    
    // Get first token after 'let' - could be type or variable name
    let first_token = &tokens[i];
    let first_string = token_to_type_string(first_token)
        .ok_or_else(|| format!("Expected identifier or type after 'let', but found: {:?}", first_token))?;
    i += 1;
    
    if i >= tokens.len() {
        return Err(format!("Unexpected end of tokens after 'let {}'", first_string));
    }
    
    match &tokens[i] {
        Token::Ident(second_ident) => {
            // Check for '=' after the second identifier
            i += 1;
            if i < tokens.len() && tokens[i] == Token::Eq {
                // Format: let type name = value
                i += 1; // skip '='
                let mut k = i;
                match parse_value(tokens, &mut k) {
                    Ok(value) => {
                        let ast_node = AST::VarDecl(first_string, second_ident.clone(), value);
                        Ok((ast_node, k))
                    }
                    Err(e) => Err(format!("Error parsing variable declaration value: {}", e)),
                }
            } else {
                Err(format!("Expected '=' after variable type '{}' and name '{}'", first_string, second_ident))
            }
        }
        Token::Eq => {
            // Format: let name = value (infer type)
            i += 1; // skip '='
            let mut k = i;
            match parse_value(tokens, &mut k) {
                Ok(value) => {
                    // Infer type from value
                    let ty = match &value {
                        ASTValue::Int(_) => "i32".to_string(),
                        ASTValue::Int64(_) => "i64".to_string(),
                        ASTValue::Float32(_) => "f32".to_string(),
                        ASTValue::Float64(_) => "f64".to_string(),
                        ASTValue::Str(_) => "string".to_string(),
                        ASTValue::Bool(_) => "bool".to_string(),
                        ASTValue::VarRef(_) => "auto".to_string(),
                        ASTValue::FuncCall { .. } => "auto".to_string(),
                        _ => "auto".to_string(),
                    };
                    let ast_node = AST::VarDecl(ty, first_string, value);
                    Ok((ast_node, k))
                }
                Err(e) => Err(format!("Error parsing variable declaration value: {}", e)),
            }
        }
        _ => {
            Err(format!("Expected identifier or '=' after 'let {}', but found: {:?}", first_string, tokens[i]))
        }
    }
}

// Helper function to parse comparison expressions
fn parse_condition(tokens: &[Token], index: &mut usize) -> Result<ASTValue, String> {
    // Check for opening parenthesis
    let has_paren = if *index < tokens.len() && tokens[*index] == Token::LParen {
        *index += 1; // skip '('
        true
    } else {
        false
    };
    
    // Parse left side
    let left = parse_value(tokens, index)?;
    
    // Check for comparison operator
    if *index >= tokens.len() {
        if has_paren {
            return Err("Expected closing ')' after condition".to_string());
        }
        return Ok(left);
    }
    
    let condition = match &tokens[*index] {
        Token::Less => {
            *index += 1;
            let right = parse_value(tokens, index)?;
            ASTValue::LessThan(Box::new(left), Box::new(right))
        }
        Token::Greater => {
            *index += 1;
            let right = parse_value(tokens, index)?;
            ASTValue::GreaterThan(Box::new(left), Box::new(right))
        }
        Token::Eq => {
            *index += 1;
            let right = parse_value(tokens, index)?;
            ASTValue::Equal(Box::new(left), Box::new(right))
        }
        Token::EqEq => {
            *index += 1;
            let right = parse_value(tokens, index)?;
            ASTValue::EqualEqual(Box::new(left),Box::new(right))
        }
        Token::NotEq => {
            *index += 1;
            let right = parse_value(tokens, index)?;
            ASTValue::NotEqual(Box::new(left), Box::new(right))
        }
        Token::LessEq => {
            *index += 1;
            let right = parse_value(tokens, index)?;
            ASTValue::LessEqual(Box::new(left), Box::new(right))
        }
        Token::GreaterEq => {
            *index += 1;
            let right = parse_value(tokens, index)?;
            ASTValue::GreaterEqual(Box::new(left), Box::new(right))
        }
        _ => left, // No comparison operator, just return the value
    };
    
    // Check for closing parenthesis if we had an opening one
    if has_paren {
        if *index >= tokens.len() || tokens[*index] != Token::RParen {
            return Err("Expected closing ')' after condition".to_string());
        }
        *index += 1; // skip ')'
    }
    
    Ok(condition)
}

// Helper function to parse if statements with elif support
fn parse_if_statement(tokens: &[Token], start_index: usize) -> Result<(AST, usize), String> {
    let mut i = start_index + 1; // skip 'if'
    
    if i >= tokens.len() {
        return Err("Unexpected end of tokens after 'if'".to_string());
    }
    
    // Parse condition (with potential parentheses)
    let condition = parse_condition(tokens, &mut i)?;
    
    // Expect '{'
    if i >= tokens.len() || tokens[i] != Token::LBrace {
        return Err("Expected '{' after if condition".to_string());
    }
    i += 1; // skip '{'
    
    // Parse then body
    let mut then_body = Vec::new();
    let mut brace_count = 1;
    
    while i < tokens.len() && brace_count > 0 {
        match &tokens[i] {
            Token::LBrace => {
                brace_count += 1;
                i += 1;
            }
            Token::RBrace => {
                brace_count -= 1;
                if brace_count > 0 {
                    i += 1;
                }
            }
            _ => {
                match parse_single_statement(tokens, &mut i) {
                    Ok(Some(node)) => then_body.push(node),
                    Ok(None) => {},
                    Err(e) => return Err(format!("Error in if body: {}", e)),
                }
            }
        }
    }
    
    if brace_count != 0 {
        return Err("Unmatched braces in if statement".to_string());
    }
    
    i += 1; // skip closing '}'
    
    // Parse elif branches
    let mut elif_branches = Vec::new();
    
    while i < tokens.len() && tokens[i] == Token::Elif {
        i += 1; // skip 'elif'
        
        // Parse elif condition
        let elif_condition = parse_condition(tokens, &mut i)?;
        
        // Expect '{'
        if i >= tokens.len() || tokens[i] != Token::LBrace {
            return Err("Expected '{' after elif condition".to_string());
        }
        i += 1; // skip '{'
        
        // Parse elif body
        let mut elif_body = Vec::new();
        let mut brace_count = 1;
        
        while i < tokens.len() && brace_count > 0 {
            match &tokens[i] {
                Token::LBrace => {
                    brace_count += 1;
                    i += 1;
                }
                Token::RBrace => {
                    brace_count -= 1;
                    if brace_count > 0 {
                        i += 1;
                    }
                }
                _ => {
                    match parse_single_statement(tokens, &mut i) {
                        Ok(Some(node)) => elif_body.push(node),
                        Ok(None) => {},
                        Err(e) => return Err(format!("Error in elif body: {}", e)),
                    }
                }
            }
        }
        
        if brace_count != 0 {
            return Err("Unmatched braces in elif statement".to_string());
        }
        
        i += 1; // skip closing '}'
        elif_branches.push((elif_condition, elif_body));
    }
    
    // Check for else
    let else_body = if i < tokens.len() && tokens[i] == Token::Else {
        i += 1; // skip 'else'
        
        if i >= tokens.len() || tokens[i] != Token::LBrace {
            return Err("Expected '{' after 'else'".to_string());
        }
        i += 1; // skip '{'
        
        let mut else_statements = Vec::new();
        let mut brace_count = 1;
        
        while i < tokens.len() && brace_count > 0 {
            match &tokens[i] {
                Token::LBrace => {
                    brace_count += 1;
                    i += 1;
                }
                Token::RBrace => {
                    brace_count -= 1;
                    if brace_count > 0 {
                        i += 1;
                    }
                }
                _ => {
                    match parse_single_statement(tokens, &mut i) {
                        Ok(Some(node)) => else_statements.push(node),
                        Ok(None) => {},
                        Err(e) => return Err(format!("Error in else body: {}", e)),
                    }
                }
            }
        }
        
        if brace_count != 0 {
            return Err("Unmatched braces in else statement".to_string());
        }
        
        i += 1; // skip closing '}'
        Some(else_statements)
    } else {
        None
    };
    
    let if_node = AST::If {
        condition,
        then_body,
        elif_branches,
        else_body,
    };
    
    Ok((if_node, i))
}

// Helper function to parse a single AST node (for reuse in function bodies and top-level)
fn parse_single_statement(tokens: &[Token], i: &mut usize) -> Result<Option<AST>, String> {
    if *i >= tokens.len() {
        return Ok(None);
    }

    match &tokens[*i] {
        Token::Let => {
            match parse_let_statement(tokens, *i) {
                Ok((ast_node, new_index)) => {
                    *i = new_index;
                    Ok(Some(ast_node))
                }
                Err(e) => Err(e),
            }
        }
        Token::Semicolon => {
            *i += 1;
            Ok(Some(AST::NewLine))
        }
        Token::Use => {
            if let Some(Token::Ident(package)) = tokens.get(*i + 1) {
                let ast_node = AST::Import(package.clone());
                *i += 2;
                Ok(Some(ast_node))
            } else {
                Err("Invalid import statement - expected package name".to_string())
            }
        }
        Token::If => {
            match parse_if_statement(tokens, *i) {
                Ok((ast_node, new_index)) => {
                    *i = new_index;
                    Ok(Some(ast_node))
                }
                Err(e) => Err(e),
            }
        }
        Token::Return => {
            *i += 1;
            if let Some(val_token) = tokens.get(*i) {
                let ret_expr = match val_token {
                    Token::Integer(n) => {
                        *i += 1;
                        AST::Return(ASTValue::Int(*n))
                    }
                    Token::Integer64(n) => {
                        *i += 1;
                        AST::Return(ASTValue::Int64(*n))
                    }
                    Token::Float32(f) => {
                        *i += 1;
                        AST::Return(ASTValue::Float32(*f))
                    }
                    Token::Float64(f) => {
                        *i += 1;
                        AST::Return(ASTValue::Float64(*f))
                    }
                    Token::DefaultFloat(f) => {
                        *i += 1;
                        AST::Return(ASTValue::Float32(*f))
                    }
                    Token::String(s) => {
                        *i += 1;
                        AST::Return(ASTValue::Str(s.clone()))
                    }
                    Token::Bool(b) => {
                        *i += 1;
                        AST::Return(ASTValue::Bool(*b))
                    }
                    Token::Ident(name) => {
                        // Check if it's a function call
                        if tokens.get(*i + 1) == Some(&Token::LParen) {
                            let mut k = *i;
                            match parse_value(tokens, &mut k) {
                                Ok(func_call_value) => {
                                    *i = k;
                                    AST::Return(func_call_value)
                                }
                                Err(_) => {
                                    *i += 1;
                                    AST::Return(ASTValue::VarRef(name.clone()))
                                }
                            }
                        } else {
                            *i += 1;
                            AST::Return(ASTValue::VarRef(name.clone()))
                        }
                    }
                    _ => return Err(format!("Unexpected token after 'return': {:?}", val_token)),
                };
                Ok(Some(ret_expr))
            } else {
                Err("Expected value after 'return'".to_string())
            }
        }
        // Handle method calls
        Token::Ident(obj) if tokens.get(*i + 1) == Some(&Token::Dot) => {
            if let Some(Token::Ident(method)) = tokens.get(*i + 2) {
                if tokens.get(*i + 3) == Some(&Token::LParen) {
                    let mut args = Vec::new();
                    let mut j = *i + 4;

                    while let Some(tok) = tokens.get(j) {
                        match tok {
                            Token::RParen => break,
                            Token::String(s) => args.push(AST::Literal(ASTValue::Str(s.clone()))),
                            Token::Integer(n) => args.push(AST::Literal(ASTValue::Int(*n))),
                            Token::Integer64(n) => args.push(AST::Literal(ASTValue::Int64(*n))),
                            Token::Float32(f) => args.push(AST::Literal(ASTValue::Float32(*f))),
                            Token::Float64(f) => args.push(AST::Literal(ASTValue::Float64(*f))),
                            Token::DefaultFloat(f) => args.push(AST::Literal(ASTValue::Float32(*f))),
                            Token::Bool(b) => args.push(AST::Literal(ASTValue::Bool(*b))),
                            Token::Ident(name) => args.push(AST::VarRef(name.clone())),
                            Token::Comma => {}
                            _ => return Err(format!("Unexpected token in method call args: {:?}", tok)),
                        }
                        j += 1;
                    }

                    if tokens.get(j) != Some(&Token::RParen) {
                        return Err("Expected closing paren for method call".to_string());
                    }
                    
                    let call_node = AST::Call {
                        object: obj.clone(),
                        method: method.clone(),
                        args,
                    };
                    *i = j + 1;
                    Ok(Some(call_node))
                } else {
                    Err(format!("Expected '(' after method {}", method))
                }
            } else {
                Err(format!("Expected method after '{}.'", obj))
            }
        }
        // Handle standalone variable references and literals
        Token::Ident(name) => {
            *i += 1;
            Ok(Some(AST::VarRef(name.clone())))
        }
        Token::String(s) => {
            *i += 1;
            Ok(Some(AST::Literal(ASTValue::Str(s.clone()))))
        }
        Token::Integer(n) => {
            *i += 1;
            Ok(Some(AST::Literal(ASTValue::Int(*n))))
        }
        Token::Integer64(n) => {
            *i += 1;
            Ok(Some(AST::Literal(ASTValue::Int64(*n))))
        }
        Token::Float32(f) => {
            *i += 1;
            Ok(Some(AST::Literal(ASTValue::Float32(*f))))
        }
        Token::Float64(f) => {
            *i += 1;
            Ok(Some(AST::Literal(ASTValue::Float64(*f))))
        }
        Token::DefaultFloat(f) => {
            *i += 1;
            Ok(Some(AST::Literal(ASTValue::Float32(*f))))
        }
        Token::Bool(b) => {
            *i += 1;
            Ok(Some(AST::Literal(ASTValue::Bool(*b))))
        }
        _ => {
            // Skip unknown tokens
            *i += 1;
            Ok(None)
        }
    }
}

pub fn parseTokens(tokens: &[Token]) -> Vec<AST> {
    let mut ast = Vec::with_capacity(tokens.len() / 3);
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            // Handle function definitions: return_type fn name(...) { ... }
            Token::Void | Token::I32Type | Token::I64Type | Token::F32Type | Token::F64Type | Token::StringType | Token::BoolType => {
                if tokens.get(i + 1) == Some(&Token::Func) {
                    // This is a function definition
                    let mut j = i;

                    // Get return type
                    let return_type = match &tokens[j] {
                        Token::I32Type => Some("i32".to_string()),
                        Token::I64Type => Some("i64".to_string()),
                        Token::F32Type => Some("f32".to_string()),
                        Token::F64Type => Some("f64".to_string()),
                        Token::Void => Some("void".to_string()),
                        Token::StringType => Some("string".to_string()),
                        Token::BoolType => Some("bool".to_string()),
                        _ => return ast, // This should not happen
                    };
                    j += 2; // skip return type and 'fn'/'func'

                    // Function name
                    let name = if let Some(Token::Ident(name)) = tokens.get(j) {
                        j += 1;
                        name.clone()
                    } else {
                        panic!("Expected function name after fn/func");
                    };

                    // Parameter list - must start with '('
                    let mut params = Vec::new();
                    if tokens.get(j) != Some(&Token::LParen) {
                        panic!("Expected '(' after function name {}", name);
                    }
                    j += 1;

                    // Parse parameters in format: type: name, type: name, ...
                    while tokens.get(j) != Some(&Token::RParen) {
                        let param_type_token = tokens.get(j).expect("Expected parameter type");
                        let param_type = token_to_type_string(param_type_token)
                            .unwrap_or_else(|| panic!("Expected parameter type, got: {:?}", param_type_token));
                        j += 1;

                        if tokens.get(j) != Some(&Token::Colon) {
                            panic!("Expected ':' after parameter type {}", param_type);
                        }
                        j += 1;

                        let param_name = if let Some(Token::Ident(n)) = tokens.get(j) {
                            j += 1;
                            n.clone()
                        } else {
                            panic!("Expected parameter name after ':'");
                        };

                        params.push((param_name, param_type));

                        if tokens.get(j) == Some(&Token::Comma) {
                            j += 1; // skip comma
                        }
                    }
                    j += 1; // skip closing RParen

                    // Body must start with '{'
                    if tokens.get(j) != Some(&Token::LBrace) {
                        panic!("Expected '{{' to start function body for {}", name);
                    }
                    j += 1;

                    // Parse body
                    let mut body = Vec::new();
                    let mut brace_count = 1;
                    while j < tokens.len() && brace_count > 0 {
                        match &tokens[j] {
                            Token::LBrace => { 
                                brace_count += 1; 
                                j += 1; 
                            }
                            Token::RBrace => { 
                                brace_count -= 1; 
                                if brace_count > 0 {
                                    j += 1;
                                }
                            }
                            _ => {
                                // Use the helper function to parse statements in function body
                                match parse_single_statement(tokens, &mut j) {
                                    Ok(Some(node)) => body.push(node),
                                    Ok(None) => {}, // Skip unhandled tokens
                                    Err(e) => panic!("Error in function body: {}", e),
                                }
                            }
                        }
                    }

                    if brace_count != 0 {
                        panic!("Unmatched braces in function {}", name);
                    }

                    ast.push(AST::FuncDef {
                        name,
                        params,
                        return_type,
                        body,
                    });

                    i = j + 1; // +1 to skip the final closing brace
                } else {
                    // Not a function definition, treat as regular token
                    match parse_single_statement(tokens, &mut i) {
                        Ok(Some(node)) => ast.push(node),
                        Ok(None) => {}, // Skip
                        Err(e) => panic!("{}", e),
                    }
                }
            }
            
            // Handle standalone 'fn' without return type
            Token::Func => {
                let mut j = i + 1; // skip 'fn'

                // Function name
                let name = if let Some(Token::Ident(name)) = tokens.get(j) {
                    j += 1;
                    name.clone()
                } else {
                    panic!("Expected function name after fn/func");
                };

                // Parameter parsing (same as above)
                let mut params = Vec::new();
                if tokens.get(j) != Some(&Token::LParen) {
                    panic!("Expected '(' after function name {}", name);
                }
                j += 1;

                while tokens.get(j) != Some(&Token::RParen) {
                    let param_type_token = tokens.get(j).expect("Expected parameter type");
                    let param_type = token_to_type_string(param_type_token)
                        .unwrap_or_else(|| panic!("Expected parameter type, got: {:?}", param_type_token));
                    j += 1;

                    if tokens.get(j) != Some(&Token::Colon) {
                        panic!("Expected ':' after parameter type {}", param_type);
                    }
                    j += 1;

                    let param_name = if let Some(Token::Ident(n)) = tokens.get(j) {
                        j += 1;
                        n.clone()
                    } else {
                        panic!("Expected parameter name after ':'");
                    };

                    params.push((param_name, param_type));

                    if tokens.get(j) == Some(&Token::Comma) {
                        j += 1;
                    }
                }
                j += 1;

                // Body parsing (same as above)
                if tokens.get(j) != Some(&Token::LBrace) {
                    panic!("Expected '{{' to start function body for {}", name);
                }
                j += 1;

                let mut body = Vec::new();
                let mut brace_count = 1;
                while j < tokens.len() && brace_count > 0 {
                    match &tokens[j] {
                        Token::LBrace => { 
                            brace_count += 1; 
                            j += 1; 
                        }
                        Token::RBrace => { 
                            brace_count -= 1; 
                            if brace_count > 0 {
                                j += 1;
                            }
                        }
                        _ => {
                            match parse_single_statement(tokens, &mut j) {
                                Ok(Some(node)) => body.push(node),
                                Ok(None) => {},
                                Err(e) => panic!("Error in function body: {}", e),
                            }
                        }
                    }
                }

                ast.push(AST::FuncDef {
                    name,
                    params,
                    return_type: None, // No explicit return type
                    body,
                });

                i = j + 1;
            }

            _ => {
                // Handle all other cases using the helper function
                match parse_single_statement(tokens, &mut i) {
                    Ok(Some(node)) => ast.push(node),
                    Ok(None) => {}, // Skip unhandled tokens
                    Err(e) => panic!("{}", e),
                }
            }
        }
    }

    ast
}

// Helper function to parse values (literals, variables, function calls)
fn parse_value(tokens: &[Token], index: &mut usize) -> Result<ASTValue, String> {
    if *index >= tokens.len() {
        return Err("Unexpected end of tokens".to_string());
    }

    match &tokens[*index] {
        Token::String(s) => {
            let value = ASTValue::Str(s.clone());
            *index += 1;
            Ok(value)
        }
        Token::Integer(n) => {
            let value = ASTValue::Int(*n);
            *index += 1;
            Ok(value)
        }
        Token::Integer64(n) => {
            let value = ASTValue::Int64(*n);
            *index += 1;
            Ok(value)
        }
        Token::Float32(f) => {
            let value = ASTValue::Float32(*f);
            *index += 1;
            Ok(value)
        }
        Token::Float64(f) => {
            let value = ASTValue::Float64(*f);
            *index += 1;
            Ok(value)
        }
        Token::DefaultFloat(f) => {
            let value = ASTValue::Float32(*f);
            *index += 1;
            Ok(value)
        }
        Token::Bool(b) => {
            let value = ASTValue::Bool(*b);
            *index += 1;
            Ok(value)
        }
        Token::Ident(name) => {
            // Check if this is a function call (identifier followed by '(')
            if tokens.get(*index + 1) == Some(&Token::LParen) {
                let func_name = name.clone();
                *index += 2; // skip function name and '('
                
                let mut args = Vec::new();
                
                // Parse arguments
                while *index < tokens.len() && tokens[*index] != Token::RParen {
                    match &tokens[*index] {
                        Token::String(s) => {
                            args.push(ASTValue::Str(s.clone()));
                            *index += 1;
                        }
                        Token::Integer(n) => {
                            args.push(ASTValue::Int(*n));
                            *index += 1;
                        }
                        Token::Integer64(n) => {
                            args.push(ASTValue::Int64(*n));
                            *index += 1;
                        }
                        Token::Float32(f) => {
                            args.push(ASTValue::Float32(*f));
                            *index += 1;
                        }
                        Token::Float64(f) => {
                            args.push(ASTValue::Float64(*f));
                            *index += 1;
                        }
                        Token::DefaultFloat(f) => {
                            args.push(ASTValue::Float32(*f));
                            *index += 1;
                        }
                        Token::Bool(b) => {
                            args.push(ASTValue::Bool(*b));
                            *index += 1;
                        }
                        Token::Ident(arg_name) => {
                            // Check if this argument is also a function call
                            if tokens.get(*index + 1) == Some(&Token::LParen) {
                                let nested_call = parse_value(tokens, index)?;
                                args.push(nested_call);
                            } else {
                                args.push(ASTValue::VarRef(arg_name.clone()));
                                *index += 1;
                            }
                        }
                        Token::Comma => {
                            *index += 1; // skip comma
                        }
                        _ => return Err(format!("Unexpected token in function arguments: {:?}", tokens[*index])),
                    }
                }
                
                if *index >= tokens.len() || tokens[*index] != Token::RParen {
                    return Err("Expected closing ')' for function call".to_string());
                }
                *index += 1; // skip ')'
                
                Ok(ASTValue::FuncCall {
                    name: func_name,
                    args,
                })
            } else {
                // It's just a variable reference
                let value = ASTValue::VarRef(name.clone());
                *index += 1;
                Ok(value)
            }
        }
        _ => Err(format!("Expected value, got: {:?}", tokens[*index])),
    }
}
