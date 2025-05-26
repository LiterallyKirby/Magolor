package main

import (
	"fmt"
	"strings"
)

// Node interface for all AST nodes
type Node interface {
	String() string
}

// Expression interface for all expression nodes
type Expression interface {
	Node
	expressionNode()
}

// Statement interface for all statement nodes
type Statement interface {
	Node
	statementNode()
}

// Program represents the root node of the AST
type Program struct {
	Statements []Statement
}

func (p *Program) String() string {
	var out strings.Builder
	for _, s := range p.Statements {
		out.WriteString(s.String())
		out.WriteString("\n")
	}
	return out.String()
}

// ============================================================================
// EXPRESSION NODES
// ============================================================================

// Identifier represents variable names
type Identifier struct {
	Token Token // the token.IDENT token
	Value string
}

func (i *Identifier) expressionNode() {}
func (i *Identifier) String() string  { return i.Value }

// IntegerLiteral represents integer values
type IntegerLiteral struct {
	Token Token
	Value int64
}

func (il *IntegerLiteral) expressionNode() {}
func (il *IntegerLiteral) String() string  { return il.Token.Literal }

// PrefixExpression represents prefix operations like -x or !x
type PrefixExpression struct {
	Token    Token // The prefix token, e.g., '!'
	Operator string
	Right    Expression
}

func (pe *PrefixExpression) expressionNode() {}
func (pe *PrefixExpression) String() string {
	return fmt.Sprintf("(%s%s)", pe.Operator, pe.Right.String())
}

// InfixExpression represents binary operations like x + y
type InfixExpression struct {
	Token    Token
	Operator string
	Left     Expression
	Right    Expression
}

func (ie *InfixExpression) expressionNode() {}
func (ie *InfixExpression) String() string {
	return fmt.Sprintf("(%s %s %s)", ie.Left, ie.Operator, ie.Right)
}

// FunctionLiteral represents function expressions
type FunctionLiteral struct {
	Token      Token // The 'fn' token
	Parameters []*Parameter
	ReturnType Token
	Body       *BlockStatement
}

func (fl *FunctionLiteral) expressionNode() {}
func (fl *FunctionLiteral) String() string {
	var params []string
	for _, p := range fl.Parameters {
		params = append(params, p.String())
	}
	return fmt.Sprintf("%s fn(%s) %s", fl.ReturnType.Literal, strings.Join(params, ", "), fl.Body.String())
}

// Parameter represents a function parameter with type and name
type Parameter struct {
	Type Token       // The type token (int, string, etc.)
	Name *Identifier // The parameter name
}

func (p *Parameter) String() string {
	return fmt.Sprintf("%s %s", p.Type.Literal, p.Name.String())
}

// TypeOfExpression represents typeof operations like typeof(x)
type TypeOfExpression struct {
	Token Token      // The 'typeof' token
	Expr  Expression // The expression to get the type of
}

func (te *TypeOfExpression) expressionNode() {}
func (te *TypeOfExpression) String() string {
	return fmt.Sprintf("typeof(%s)", te.Expr.String())
}

// ============================================================================
// STATEMENT NODES
// ============================================================================

// Variable declaration
type VariableDeclaration struct {
	Token Token
	Name  *Identifier
	Value Expression
}

func (vd *VariableDeclaration) statementNode() {}

func (vd *VariableDeclaration) String() string {
	return fmt.Sprintf("%s %s = %s;", vd.Token.Literal, vd.Name.String(), vd.Value.String())
}

type VoidLiteral struct {
	Token Token
}

func (vl *VoidLiteral) expressionNode() {}
func (vl *VoidLiteral) String() string  { return vl.Token.Literal }

// ReturnStatement represents return statements
type ReturnStatement struct {
	Token       Token
	ReturnValue Expression
}

func (rs *ReturnStatement) statementNode() {}
func (rs *ReturnStatement) String() string {
	if rs.ReturnValue != nil {
		return fmt.Sprintf("return %s;", rs.ReturnValue.String())
	}
	return "return;"
}

// BlockStatement represents code blocks
type BlockStatement struct {
	Token      Token // the '{' token
	Statements []Statement
}

func (bs *BlockStatement) statementNode() {}
func (bs *BlockStatement) String() string {
	var out strings.Builder
	out.WriteString("{ ")
	for _, s := range bs.Statements {
		out.WriteString(s.String())
		out.WriteString(" ")
	}
	out.WriteString("}")
	return out.String()
}

type IfStatement struct {
	Token     Token
	Condition Expression
	ThenBlock *BlockStatement
	ElseIfs   []ElseIfClause // Changed to use ElseIfClause type
	ElseBlock *BlockStatement
}

func (is *IfStatement) String() string {
	var out strings.Builder

	// 1. if clause
	out.WriteString("if (")
	out.WriteString(is.Condition.String())
	out.WriteString(") ")
	out.WriteString(is.ThenBlock.String())

	// 2. else if clauses
	for _, elif := range is.ElseIfs {
		out.WriteString(" else if (")
		out.WriteString(elif.Condition.String())
		out.WriteString(") ")
		out.WriteString(elif.Block.String())
	}

	// 3. else clause
	if is.ElseBlock != nil {
		out.WriteString(" else ")
		out.WriteString(is.ElseBlock.String())
	}

	return out.String()
}

func (is *IfStatement) statementNode() {}

type ElseIfClause struct {
	Condition Expression
	Block     *BlockStatement
}
type ElseIfBranch struct {
	Condition Expression // Condition for this "else if"
	Block     *Block     // Code block for this "else if"
}

type Block struct {
	Statements []Statement // List of statements in this block
}

type LoopStatement struct {
	Token Token
	Block *BlockStatement
}

func (ls *LoopStatement) statementNode() {}

func (ls *LoopStatement) TokenLiteral() string { return ls.Token.Literal }

func (ls *LoopStatement) String() string {
	if ls.Block == nil{
		return "loop (missing block)"
	}

	return "loop " + ls.Block.String()

}

type WhileStatement struct {
	Token     Token
	Block     *BlockStatement
	Condition Expression
}

func (ws *WhileStatement) statementNode() {}

func (ls *WhileStatement) TokenLiteral() string { return ls.Token.Literal }

func (ws *WhileStatement) String() string {
	if ws.Block == nil {
		return "while (missing block)"
	}
	return "while (" + ws.Condition.String() + ") " + ws.Block.String()
}


type ForStatement struct {
	Token     Token
	Identifier *Identifier
	Iterable   Expression
	Block      *BlockStatement
}

func (fs *ForStatement) statementNode() {}

func (ls *ForStatement) TokenLiteral() string { return ls.Token.Literal }

func (fs *ForStatement) String() string {
	iter := ""
	if fs.Identifier != nil {
		iter += fs.Identifier.String()
	}
	iter += " in "
	if fs.Iterable != nil {
		iter += fs.Iterable.String()
	}
	block := ""
	if fs.Block != nil {
		block = fs.Block.String()
	}
	return "for (" + iter + ") " + block
}


// FunctionStatement represents function declarations
type FunctionStatement struct {
	Token      Token // the return type token
	Name       *Identifier
	Parameters []*Parameter
	ReturnType Token
	Body       *BlockStatement
}

func (fs *FunctionStatement) statementNode() {}
func (fs *FunctionStatement) String() string {
	var params []string
	for _, p := range fs.Parameters {
		params = append(params, p.String())
	}
	return fmt.Sprintf("%s %s(%s) %s", fs.ReturnType.Literal, fs.Name.String(), strings.Join(params, ", "), fs.Body.String())
}


