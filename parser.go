package main

import (
	"fmt"
	"strconv"
)

// Parser struct to hold lexer and tokens for parsing
type Parser struct {
	lexer     *Lexer
	curToken  Token
	peekToken Token
	peekPeek  Token
	errors    []string
}

// NewParser creates a parser and reads tokens to initialize
func NewParser(l *Lexer) *Parser {
	p := &Parser{lexer: l}
	p.nextToken()
	p.nextToken()
	p.nextToken() // Load peekPeek as well
	return p
}

func (p *Parser) peekTokenIs(t TokenType) bool {
	return p.peekToken.Type == t
}

// nextToken advances tokens
func (p *Parser) nextToken() {
	p.curToken = p.peekToken
	p.peekToken = p.peekPeek
	p.peekPeek = p.lexer.NextToken()
}

// addError adds an error message to the parser's error list
func (p *Parser) addError(msg string) {
	p.errors = append(p.errors, msg)
}

// expectPeek checks if the next token matches expected type and advances if so
func (p *Parser) expectPeek(t TokenType) bool {
	if p.peekToken.Type == t {
		p.nextToken()
		return true
	}
	p.addError(fmt.Sprintf("expected next token to be %s, got %s instead", t, p.peekToken.Type))
	return false
}

// Precedence helper functions
func (p *Parser) peekPrecedence() int {
	if prec, ok := precedences[p.peekToken.Type]; ok {
		return prec
	}
	return LOWEST
}

func (p *Parser) curPrecedence() int {
	if prec, ok := precedences[p.curToken.Type]; ok {
		return prec
	}
	return LOWEST
}

// skipWhitespaceTokens skips any whitespace or empty tokens
func (p *Parser) skipWhitespaceTokens() {
	for p.curToken.Type == ILLEGAL && p.curToken.Literal == "" {
		p.nextToken()
		if p.curToken.Type == EOF {
			break
		}
	}
}

// ============================================================================
// PARSING METHODS
// ============================================================================

func (p *Parser) parseExpression(precedence int) Expression {
	leftExp := p.parsePrefixExpression()
	if leftExp == nil {
		return nil
	}

	for p.peekToken.Type != SEMICOLON && precedence < p.peekPrecedence() && p.peekToken.Type != EOF {
		p.nextToken()
		leftExp = p.parseInfixExpression(leftExp)
		if leftExp == nil {
			return nil
		}
	}

	return leftExp
}

func (p *Parser) parsePrefixExpression() Expression {
	// Skip any whitespace tokens first
	p.skipWhitespaceTokens()

	switch p.curToken.Type {
	case VOID:
		return &VoidLiteral{Token: p.curToken}
	case IDENT:
		return &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	case INT:
		value, err := strconv.ParseInt(p.curToken.Literal, 10, 64)
		if err != nil {
			p.addError(fmt.Sprintf("could not parse %q as integer", p.curToken.Literal))
			return nil
		}
		return &IntegerLiteral{Token: p.curToken, Value: value}

	case BOOL:
		return p.parseBooleanLiteral()
	case NIL:
		return &NilLiteral{Token: p.curToken}
	case STRING:
    return &StringLiteral{Token: p.curToken, Value: p.curToken.Literal}
	case FLOAT:
		value, err := strconv.ParseFloat(p.curToken.Literal, 64)
		if err != nil {
			p.addError(fmt.Sprintf("could not parse %q as float", p.curToken.Literal))
			return nil
		}
		lit := &FloatLiteral{Token: p.curToken, Value: value}
		p.nextToken()
		return lit
	case LPAREN:
		p.nextToken()
		exp := p.parseExpression(LOWEST)
		if !p.expectPeek(RPAREN) {
			return nil
		}
		return exp
	case SUB, ADD: // Handle prefix operators like -5 or +5
		return p.parsePrefixOperatorExpression()
	case TYPEOF: // Handle typeof expressions
		return p.parseTypeofExpression()
	default:
		if p.curToken.Type != EOF {
			p.addError(fmt.Sprintf("no prefix parse function for %s found", p.curToken.Type))
		}
		return nil
	}
}

func (p *Parser) parseBooleanLiteral() Expression {
	return &BooleanLiteral{
		Token: p.curToken,
		Value: p.curToken.Literal == "true",
	}
}

func (p *Parser) parsePrefixOperatorExpression() Expression {
	expression := &PrefixExpression{
		Token:    p.curToken,
		Operator: p.curToken.Literal,
	}

	p.nextToken()
	expression.Right = p.parseExpression(PREFIX)

	return expression
}

func (p *Parser) parseFloatLiteral() Expression {
	lit := &FloatLiteral{Token: p.curToken}
	value, err := strconv.ParseFloat(p.curToken.Literal, 64)
	if err != nil {
		p.errors = append(p.errors, "could not parse float")
		return nil
	}
	lit.Value = value
	p.nextToken()
	return lit
}

func (p *Parser) parseStringLiteral() Expression {
	lit := &StringLiteral{Token: p.curToken, Value: p.curToken.Literal}
	p.nextToken()
	return lit
}

func (p *Parser) parseInfixExpression(left Expression) Expression {
	expression := &InfixExpression{
		Token:    p.curToken,
		Operator: p.curToken.Literal,
		Left:     left,
	}

	precedence := p.curPrecedence()
	p.nextToken()
	expression.Right = p.parseExpression(precedence)

	return expression
}

func (p *Parser) parseReturnStatement() *ReturnStatement {
	stmt := &ReturnStatement{Token: p.curToken}

	// Look ahead to see what comes next
	if p.peekToken.Type == SEMICOLON {
		// Empty return with semicolon
		p.nextToken() // consume semicolon
		return stmt
	} else if p.peekToken.Type == RBRACE {
		// Empty return at end of block
		return stmt
	}

	p.nextToken() // move to the expression after 'return'
	stmt.ReturnValue = p.parseExpression(LOWEST)

	// Consume semicolon if it exists
	if p.peekToken.Type == SEMICOLON {
		p.nextToken()
	}

	return stmt
}

func (p *Parser) parseBreakStatement() *BreakStatement {
	stmt := &BreakStatement{Token: p.curToken}

	// Consume semicolon if present
	if p.peekToken.Type == SEMICOLON {
		p.nextToken()
	}

	return stmt
}

func (p *Parser) parseContinueStatement() *ContinueStatement {
	stmt := &ContinueStatement{Token: p.curToken}

	// Consume semicolon if present
	if p.peekToken.Type == SEMICOLON {
		p.nextToken()
	}

	return stmt
}

func (p *Parser) curTokenIs(t TokenType) bool {
	return p.curToken.Type == t
}

func (p *Parser) parseBlockStatement() *BlockStatement {
	block := &BlockStatement{Token: p.curToken}
	p.nextToken() // consume '{'

	block.Statements = []Statement{}

	for !p.curTokenIs(RBRACE) && !p.curTokenIs(EOF) {
		stmt := p.parseStatement()
		if stmt != nil {
			block.Statements = append(block.Statements, stmt)
		}
		p.nextToken()
	}

	return block
}

// Fixed parseStatement function
func (p *Parser) parseStatement() Statement {
	p.skipWhitespaceTokens()

	switch p.curToken.Type {
	case IF:
		return p.parseIfStatement()
	case RETURN:
		return p.parseReturnStatement()
	case BREAK:
		return p.parseBreakStatement()
	case CONTINUE:
		return p.parseContinueStatement()
	case WHILE:
		return p.parseWhileStatement()
	case LOOP:
		return p.parseLoopStatement()
	case FOR:
		return p.parseForStatement()
	case FUNC:
		return p.parseFunctionStatement()
	case TYPE, VOID:
		// Look ahead to determine if this is a function or variable declaration
		if p.peekToken.Type == IDENT {
			if p.peekPeek.Type == LPAREN {
				return p.parseFunctionStatement()
			} else if p.peekPeek.Type == ASSIGN {
				return p.parseDeclarationStatement()
			}
			// fallback for declaration without assignment
			return p.parseDeclarationStatement()
		}
	case RBRACE, SEMICOLON, EOF:
		return nil
	default:
		// Try to parse as expression statement
		expr := p.parseExpression(LOWEST)
		if expr != nil {
			stmt := &ExpressionStatement{Token: p.curToken, Expression: expr}
			// Consume semicolon if present
			if p.peekToken.Type == SEMICOLON {
				p.nextToken()
			}
			return stmt
		}
		if p.curToken.Type != EOF {
			p.addError(fmt.Sprintf("unexpected token: %s", p.curToken.Literal))
		}
		return nil
	}
	return nil
}

func (p *Parser) parseDeclarationStatement() *VariableDeclaration {
	if p.curToken.Type != TYPE && p.curToken.Type != VOID {
		p.addError(fmt.Sprintf("expected type, got %s", p.curToken.Type))
		return nil
	}

	token := p.curToken

	if !p.expectPeek(IDENT) {
		return nil
	}

	name := &Identifier{Token: p.curToken, Value: p.curToken.Literal}

	if !p.expectPeek(ASSIGN) {
		return nil
	}

	p.nextToken() // Move to the expression
	value := p.parseExpression(LOWEST)

	if p.peekTokenIs(SEMICOLON) {
		p.nextToken()
	}

	return &VariableDeclaration{
		Token: token,
		Name:  name,
		Value: value,
	}
}

func (p *Parser) parseWhileStatement() *WhileStatement {
	stmt := &WhileStatement{Token: p.curToken}

	if !p.expectPeek(LPAREN) {
		p.addError("expected '(' after 'while'")
		return nil
	}

	p.nextToken() // move past '('
	stmt.Condition = p.parseExpression(LOWEST)
	if stmt.Condition == nil {
		return nil
	}

	if !p.expectPeek(RPAREN) {
		p.addError("expected ')' after while condition")
		return nil
	}

	if !p.expectPeek(LBRACE) {
		p.addError("expected '{' after while condition")
		return nil
	}

	stmt.Block = p.parseBlockStatement()
	return stmt
}

func (p *Parser) parseLoopStatement() *LoopStatement {
	stmt := &LoopStatement{Token: p.curToken}

	if !p.expectPeek(LBRACE) {
		p.addError("expected '{' after 'loop'")
		return nil
	}

	stmt.Block = p.parseBlockStatement()
	return stmt
}

func (p *Parser) parseForStatement() *ForStatement {
	stmt := &ForStatement{Token: p.curToken}

	if !p.expectPeek(LPAREN) {
		p.addError("expected '(' after 'for'")
		return nil
	}

	if !p.expectPeek(IDENT) {
		p.addError("expected identifier after '(' in for statement")
		return nil
	}
	stmt.Identifier = &Identifier{Token: p.curToken, Value: p.curToken.Literal}

	if !p.expectPeek(IN) {
		p.addError("expected 'in' after identifier in for statement")
		return nil
	}

	p.nextToken() // move past 'in'
	stmt.Iterable = p.parseExpression(LOWEST)
	if stmt.Iterable == nil {
		return nil
	}

	if !p.expectPeek(RPAREN) {
		p.addError("expected ')' after for loop iterable")
		return nil
	}

	if !p.expectPeek(LBRACE) {
		p.addError("expected '{' after for loop header")
		return nil
	}

	stmt.Block = p.parseBlockStatement()
	return stmt
}

func (p *Parser) parseFunctionStatement() *FunctionStatement {
	stmt := &FunctionStatement{}

	// Handle both "func name()" and "type name()" syntax
	if p.curToken.Type == FUNC {
		stmt.Token = p.curToken
		// For "func" keyword, we assume void return type unless specified differently
		stmt.ReturnType = Token{Type: VOID, Literal: "void"}

		if !p.expectPeek(IDENT) {
			return nil
		}
		stmt.Name = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	} else {
		// Handle "type name()" syntax
		stmt.ReturnType = p.curToken // Can be TYPE or VOID
		stmt.Token = p.curToken

		if !p.expectPeek(IDENT) {
			return nil
		}
		stmt.Name = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	}

	// Expect opening parenthesis
	if !p.expectPeek(LPAREN) {
		return nil
	}

	// Parse parameters
	stmt.Parameters = p.parseFunctionParameters()

	// Expect opening brace
	if !p.expectPeek(LBRACE) {
		return nil
	}

	// Parse function body
	stmt.Body = p.parseBlockStatement()

	// parseBlockStatement leaves us at the closing RBRACE
	// Don't advance here - let the main parsing loop handle advancement
	return stmt
}

func (p *Parser) parseFunctionParameters() []*Parameter {
	var params []*Parameter

	// Handle empty parameter list
	if p.peekToken.Type == RPAREN {
		p.nextToken()
		return params
	}

	// Parse first parameter
	p.nextToken()
	param := p.parseParameter()
	if param != nil {
		params = append(params, param)
	}

	// Parse remaining parameters
	for p.peekToken.Type == COMMA {
		p.nextToken() // consume comma
		p.nextToken() // move to next parameter
		param := p.parseParameter()
		if param != nil {
			params = append(params, param)
		}
	}

	if !p.expectPeek(RPAREN) {
		return nil
	}

	return params
}

func (p *Parser) parseParameter() *Parameter {
	// Expect: TYPE IDENT (e.g., "int x")
	if p.curToken.Type != TYPE {
		p.addError(fmt.Sprintf("expected parameter type, got %s", p.curToken.Type))
		return nil
	}

	param := &Parameter{Type: p.curToken}

	if !p.expectPeek(IDENT) {
		return nil
	}

	param.Name = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	return param
}

func (p *Parser) parseTypeofExpression() Expression {
	expression := &TypeOfExpression{Token: p.curToken}

	// Expect opening parenthesis
	if !p.expectPeek(LPAREN) {
		return nil
	}

	// Move to the expression inside parentheses
	p.nextToken()
	expression.Expr = p.parseExpression(LOWEST)

	// Expect closing parenthesis
	if !p.expectPeek(RPAREN) {
		return nil
	}

	return expression
}

func (p *Parser) parseIfStatement() *IfStatement {
	stmt := &IfStatement{Token: p.curToken}

	// Parse condition
	if !p.expectPeek(LPAREN) {
		return nil
	}
	p.nextToken()
	stmt.Condition = p.parseExpression(LOWEST)
	if stmt.Condition == nil {
		return nil
	}
	if !p.expectPeek(RPAREN) {
		return nil
	}

	// Parse then block - support both braced and single statement
	if p.peekToken.Type == LBRACE {
		// Braced block
		if !p.expectPeek(LBRACE) {
			return nil
		}
		stmt.ThenBlock = p.parseBlockStatement()
	} else {
		// Single statement - create a synthetic block
		p.nextToken()
		singleStmt := p.parseStatement()
		if singleStmt == nil {
			return nil
		}
		stmt.ThenBlock = &BlockStatement{
			Token:      p.curToken,
			Statements: []Statement{singleStmt},
		}
	}

	// Handle else/else-if clauses
	for p.peekToken.Type == ELSE {
		p.nextToken() // consume 'else'

		if p.peekToken.Type == IF {
			p.nextToken() // consume 'if'

			// Parse else-if condition
			if !p.expectPeek(LPAREN) {
				return nil
			}
			p.nextToken()
			condition := p.parseExpression(LOWEST)
			if condition == nil {
				return nil
			}
			if !p.expectPeek(RPAREN) {
				return nil
			}

			// Parse else-if block - support both braced and single statement
			var block *BlockStatement
			if p.peekToken.Type == LBRACE {
				// Braced block
				if !p.expectPeek(LBRACE) {
					return nil
				}
				block = p.parseBlockStatement()
			} else {
				// Single statement - create a synthetic block
				p.nextToken()
				singleStmt := p.parseStatement()
				if singleStmt == nil {
					return nil
				}
				block = &BlockStatement{
					Token:      p.curToken,
					Statements: []Statement{singleStmt},
				}
			}

			// Add to ElseIfs
			stmt.ElseIfs = append(stmt.ElseIfs, ElseIfClause{
				Condition: condition,
				Block:     block,
			})
		} else {
			// Parse else block - support both braced and single statement
			if p.peekToken.Type == LBRACE {
				// Braced block
				if !p.expectPeek(LBRACE) {
					return nil
				}
				stmt.ElseBlock = p.parseBlockStatement()
			} else {
				// Single statement - create a synthetic block
				p.nextToken()
				singleStmt := p.parseStatement()
				if singleStmt == nil {
					return nil
				}
				stmt.ElseBlock = &BlockStatement{
					Token:      p.curToken,
					Statements: []Statement{singleStmt},
				}
			}
			break // No more else/else-if after a plain else
		}
	}

	return stmt
}

func (p *Parser) ParseProgram() *Program {
	program := &Program{}

	for p.curToken.Type != EOF {
		p.skipWhitespaceTokens()
		if p.curToken.Type == EOF {
			break
		}

		stmt := p.parseStatement()
		if stmt != nil {
			program.Statements = append(program.Statements, stmt)
		}

		// Advance to next token
		p.nextToken()
	}

	return program
}

// ExpressionStatement represents expressions used as statements
type ExpressionStatement struct {
	Token      Token
	Expression Expression
}

func (es *ExpressionStatement) statementNode() {}
func (es *ExpressionStatement) String() string {
	if es.Expression != nil {
		return es.Expression.String()
	}
	return ""
}

// Additional AST node types for new functionality

// BooleanLiteral represents boolean literals (true/false)
type BooleanLiteral struct {
	Token Token
	Value bool
}

func (bl *BooleanLiteral) expressionNode() {}
func (bl *BooleanLiteral) String() string {
	return bl.Token.Literal
}

// NilLiteral represents null/nil literals
type NilLiteral struct {
	Token Token
}

func (nl *NilLiteral) expressionNode() {}
func (nl *NilLiteral) String() string {
	return nl.Token.Literal
}

// StringLiteral represents string literals
type StringLiteral struct {
	Token Token
	Value string
}

func (sl *StringLiteral) expressionNode() {}
func (sl *StringLiteral) String() string {
    return fmt.Sprintf("\"%s\"", sl.Value)
}

// FloatLiteral represents floating point literals
type FloatLiteral struct {
	Token Token
	Value float64
}

func (fl *FloatLiteral) expressionNode() {}
func (fl *FloatLiteral) String() string {
	return fl.Token.Literal
}

// BreakStatement represents break statements
type BreakStatement struct {
	Token Token
}

func (bs *BreakStatement) statementNode() {}
func (bs *BreakStatement) String() string {
	return "break;"
}

// ContinueStatement represents continue statements
type ContinueStatement struct {
	Token Token
}

func (cs *ContinueStatement) statementNode() {}
func (cs *ContinueStatement) String() string {
	return "continue;"
}

// GetErrors returns all parsing errors
func (p *Parser) GetErrors() []string {
	return p.errors
}
