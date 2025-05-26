package main

// TokenType is a simple string alias for token categories
type TokenType string

const (
	// Special tokens
	ILLEGAL TokenType = "ILLEGAL"
	EOF     TokenType = "EOF"

	// Identifiers and literals
	IDENT  TokenType = "IDENT" // main, foo, x, etc.
	INT    TokenType = "INT"   // 123, 42
	STRING TokenType = "STRING"
	FLOAT  TokenType = "FLOAT"
	BOOL   TokenType = "BOOL" // true, false
	NIL    TokenType = "NIL"  // null, nil

	// Delimiters
	LPAREN    TokenType = "("
	RPAREN    TokenType = ")"
	LBRACE    TokenType = "{"
	RBRACE    TokenType = "}"
	COMMA     TokenType = ","
	SEMICOLON TokenType = ";"

	// Operators
	ADD    TokenType = "+"
	SUB    TokenType = "-"
	MUL    TokenType = "*"
	DIV    TokenType = "/"
	ASSIGN TokenType = "="

	// Comparison operators
	EQ     TokenType = "=="
	NOT_EQ TokenType = "!="
	LT     TokenType = "<"
	GT     TokenType = ">"
	// Additional operators you might want
	NOT TokenType = "!"  // logical not
	AND TokenType = "&&" // logical and
	OR  TokenType = "||" // logical or
	LE  TokenType = "<=" // less than or equal
	GE  TokenType = ">=" // greater than or equal
	MOD TokenType = "%"  // modulo

	// Keywords
	IF       TokenType = "if"
	ELSE     TokenType = "else"
	WHILE    TokenType = "while"
	FOR      TokenType = "for"
	LOOP     TokenType = "loop"
	IN       TokenType = "in"
	FUNC     TokenType = "fn"
	RETURN   TokenType = "return"
	TYPE     TokenType = "TYPE"
	VOID     TokenType = "void" // Move this here
	TYPEOF   TokenType = "typeof"
	BREAK    TokenType = "break"
	CONTINUE TokenType = "continue"
)

// Token represents a token with type and literal string value
type Token struct {
	Type    TokenType
	Literal string
}

// Precedence constants
const (
	_ int = iota
	LOWEST
	OR_PREC         // ||
	AND_PREC        // &&
	EQUALS          // ==
	LESSGREATER     // > or
	SUM             // +
	PRODUCT         // *
	PREFIX          // -X or !X
	CALL            // myFunction(X)
)

var precedences = map[TokenType]int{
    OR:     OR_PREC,     // ||
    AND:    AND_PREC,    // &&
    EQ:     EQUALS,      // ==
    NOT_EQ: EQUALS,      // !=
    LT:     LESSGREATER, //
    GT:     LESSGREATER, // >
    LE:     LESSGREATER, // <=
    GE:     LESSGREATER, // >=
    ADD:    SUM,         // +
    SUB:    SUM,         // -
    MUL:    PRODUCT,     // *
    DIV:    PRODUCT,     // /
    MOD:    PRODUCT,     // %
    LPAREN: CALL,        // (
}
