package main

import (
	"strings"
	"unicode"
)

// Lexer holds the state while scanning input
type Lexer struct {
	input        string
	position     int  // current char position
	readPosition int  // next char position
	ch           byte // current char
}

// NewLexer initializes Lexer with input string
func NewLexer(input string) *Lexer {
	l := &Lexer{input: input}
	l.readChar()
	return l
}

// readChar advances the lexer positions and sets current char
func (l *Lexer) readChar() {
	if l.readPosition >= len(l.input) {
		l.ch = 0 // EOF
	} else {
		l.ch = l.input[l.readPosition]
	}
	l.position = l.readPosition
	l.readPosition++
}

// peekChar returns the next character without advancing position
func (l *Lexer) peekChar() byte {
	if l.readPosition >= len(l.input) {
		return 0
	}
	return l.input[l.readPosition]
}

// skipWhitespace skips whitespace characters
func (l *Lexer) skipWhitespace() {
	for l.ch == ' ' || l.ch == '\t' || l.ch == '\n' || l.ch == '\r' {
		l.readChar()
	}
}

// readIdentifier reads an identifier or keyword
func (l *Lexer) readIdentifier() string {
	start := l.position
	for isLetter(l.ch) {
		l.readChar()
	}
	return l.input[start:l.position]
}

// readNumber reads an integer or float literal
func (l *Lexer) readNumber() string {
	start := l.position
	for isDigit(l.ch) {
		l.readChar()
	}

	if l.ch == '.' {
		l.readChar()
		for isDigit(l.ch) {
			l.readChar()
		}
	}

	return l.input[start:l.position]
}

// readString reads a string literal (supports basic, no escapes)
func (l *Lexer) readString() string {
	position := l.position + 1 // skip starting quote
	for {
		l.readChar()
		if l.ch == '"' || l.ch == 0 {
			break
		}
	}
	return l.input[position:l.position]
}

// NextToken returns the next token from input
func (l *Lexer) NextToken() Token {
	var tok Token
	l.skipWhitespace()

	switch l.ch {
	case '(':
		tok = Token{Type: LPAREN, Literal: string(l.ch)}
	case ')':
		tok = Token{Type: RPAREN, Literal: string(l.ch)}
	case '{':
		tok = Token{Type: LBRACE, Literal: string(l.ch)}
	case '}':
		tok = Token{Type: RBRACE, Literal: string(l.ch)}
	case ',':
		tok = Token{Type: COMMA, Literal: string(l.ch)}
	case ';':
		tok = Token{Type: SEMICOLON, Literal: string(l.ch)}
	case '*':
		tok = Token{Type: MUL, Literal: string(l.ch)}
	case '-':
		tok = Token{Type: SUB, Literal: string(l.ch)}
	case '+':
		tok = Token{Type: ADD, Literal: string(l.ch)}
	case '/':
		tok = Token{Type: DIV, Literal: string(l.ch)}
	case '<':
		tok = Token{Type: LT, Literal: string(l.ch)}
	case '>':
		tok = Token{Type: GT, Literal: string(l.ch)}
	case '=':
		if l.peekChar() == '=' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: EQ, Literal: string(ch) + string(l.ch)}
		} else {
			tok = Token{Type: ASSIGN, Literal: string(l.ch)}
		}
	case '!':
		if l.peekChar() == '=' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: NOT_EQ, Literal: string(ch) + string(l.ch)}
		} else {
			tok = Token{Type: NOT, Literal: string(l.ch)}
		}
	case '&':
		if l.peekChar() == '&' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: AND, Literal: string(ch) + string(l.ch)}
		} else {
			tok = Token{Type: ILLEGAL, Literal: string(l.ch)}
		}
	case '|':
		if l.peekChar() == '|' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: OR, Literal: string(ch) + string(l.ch)}
		} else {
			tok = Token{Type: ILLEGAL, Literal: string(l.ch)}
		}
	case '%':
		tok = Token{Type: MOD, Literal: string(l.ch)}
	case '"':
		tok.Type = STRING
		tok.Literal = l.readString()
	case 0:
		tok.Literal = ""
		tok.Type = EOF
	default:
		if isLetter(l.ch) {
			lit := l.readIdentifier()
			tokType := lookupIdent(lit)
			return Token{Type: tokType, Literal: lit}
		} else if isDigit(l.ch) {
			lit := l.readNumber()
			if strings.Contains(lit, ".") {
				return Token{Type: FLOAT, Literal: lit}
			}
			return Token{Type: INT, Literal: lit}
		} else {
			tok = Token{Type: ILLEGAL, Literal: string(l.ch)}
		}
	}

	l.readChar()
	return tok
}

// Helper functions for lexer
func isLetter(ch byte) bool {
	return unicode.IsLetter(rune(ch)) || ch == '_'
}

func isDigit(ch byte) bool {
	return '0' <= ch && ch <= '9'
}

func lookupIdent(ident string) TokenType {
	switch ident {
	case "if":
		return IF
	case "else":
		return ELSE
	case "int", "string", "void", "float":
		return TYPE
	case "return":
		return RETURN
	case "fn", "func":
		return FUNC
	case "typeof":
		return TYPEOF
	case "loop":
		return LOOP
	case "while":
		return WHILE
	case "for":
		return FOR
	case "in":
		return IN
	case "true", "false":
		return BOOL
	case "null", "nil":
		return NIL
	case "break":
		return BREAK
	case "continue":
		return CONTINUE
	default:
		return IDENT
	}
}
