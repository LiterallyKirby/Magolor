package main

import (
	"fmt"
	"strconv"
)

// Object interface for all evaluated values
type Object interface {
	Type() Type
	Inspect() string
}

// ObjectType represents the type of objects in our system
type ObjectType string

const (
	INTEGER_OBJ ObjectType = "INTEGER"
	STRING_OBJ  ObjectType = "STRING"
	ERROR_OBJ   ObjectType = "ERROR"
	NULL_OBJ    ObjectType = "NULL"
)

// Integer object
type Integer struct {
	Value int64
}

func (i *Integer) Inspect() string { return fmt.Sprintf("%d", i.Value) }
func (i *Integer) Type() Type      { return IntType }

// String object
type String struct {
	Value string
}

func (s *String) Inspect() string { return s.Value }
func (s *String) Type() Type      { return StringType }

// Error object
type Error struct {
	Message string
}

func (e *Error) Inspect() string { return "ERROR: " + e.Message }
func (e *Error) Type() Type      { return UnknownType }

// Null object
type Null struct{}

func (n *Null) Inspect() string { return "null" }
func (n *Null) Type() Type      { return VoidType }

// Evaluation environment for variables
type EvalEnv struct {
	store map[string]Object
	outer *EvalEnv
}

func NewEvalEnv() *EvalEnv {
	return &EvalEnv{
		store: make(map[string]Object),
		outer: nil,
	}
}

func NewEnclosedEnv(outer *EvalEnv) *EvalEnv {
	env := NewEvalEnv()
	env.outer = outer
	return env
}

func (e *EvalEnv) Get(name string) (Object, bool) {
	value, ok := e.store[name]
	if !ok && e.outer != nil {
		value, ok = e.outer.Get(name)
	}
	return value, ok
}

func (e *EvalEnv) Set(name string, val Object) Object {
	e.store[name] = val
	return val
}

// Evaluator handles expression evaluation
type Evaluator struct{}

func NewEvaluator() *Evaluator {
	return &Evaluator{}
}

// Eval evaluates expressions and returns objects
func (eval *Evaluator) Eval(node Expression, env *EvalEnv) Object {
	switch node := node.(type) {
	case *IntegerLiteral:
		return &Integer{Value: node.Value}

	case *Identifier:
		return eval.evalIdentifier(node, env)

	case *PrefixExpression:
		right := eval.Eval(node.Right, env)
		if isError(right) {
			return right
		}
		return eval.evalPrefixExpression(node.Operator, right)

	case *InfixExpression:
		left := eval.Eval(node.Left, env)
		if isError(left) {
			return left
		}
		right := eval.Eval(node.Right, env)
		if isError(right) {
			return right
		}
		return eval.evalInfixExpression(node.Operator, left, right)

	case *TypeOfExpression:
		return eval.typeof(node, env)

	default:
		return newError("unknown expression type: %T", node)
	}
}

func (eval *Evaluator) evalIdentifier(node *Identifier, env *EvalEnv) Object {
	val, ok := env.Get(node.Value)
	if !ok {
		return newError("identifier not found: " + node.Value)
	}
	return val
}

func (eval *Evaluator) evalPrefixExpression(operator string, right Object) Object {
	switch operator {
	case "-":
		return eval.evalMinusPrefixOperatorExpression(right)
	case "+":
		return eval.evalPlusPrefixOperatorExpression(right)
	default:
		return newError("unknown operator: %s%T", operator, right)
	}
}

func (eval *Evaluator) evalMinusPrefixOperatorExpression(right Object) Object {
	if right.Type() != IntType {
		return newError("unknown operator: -%T", right)
	}

	value := right.(*Integer).Value
	return &Integer{Value: -value}
}

func (eval *Evaluator) evalPlusPrefixOperatorExpression(right Object) Object {
	if right.Type() != IntType {
		return newError("unknown operator: +%T", right)
	}

	return right // +x is just x for integers
}

func (eval *Evaluator) evalInfixExpression(operator string, left, right Object) Object {
	switch {
	case left.Type() == IntType && right.Type() == IntType:
		return eval.evalIntegerInfixExpression(operator, left, right)
	case operator == "==":
		return nativeBoolToPyMonkeyBoolean(left == right)
	case operator == "!=":
		return nativeBoolToPyMonkeyBoolean(left != right)
	default:
		return newError("unknown operator: %T %s %T", left, operator, right)
	}
}

func (eval *Evaluator) evalIntegerInfixExpression(operator string, left, right Object) Object {
	leftVal := left.(*Integer).Value
	rightVal := right.(*Integer).Value

	switch operator {
	case "+":
		return &Integer{Value: leftVal + rightVal}
	case "-":
		return &Integer{Value: leftVal - rightVal}
	case "*":
		return &Integer{Value: leftVal * rightVal}
	case "/":
		if rightVal == 0 {
			return newError("division by zero")
		}
		return &Integer{Value: leftVal / rightVal}
	case "<":
		return nativeBoolToPyMonkeyBoolean(leftVal < rightVal)
	case ">":
		return nativeBoolToPyMonkeyBoolean(leftVal > rightVal)
	case "==":
		return nativeBoolToPyMonkeyBoolean(leftVal == rightVal)
	case "!=":
		return nativeBoolToPyMonkeyBoolean(leftVal != rightVal)
	default:
		return newError("unknown operator: %s", operator)
	}
}

func (eval *Evaluator) typeof(node *TypeOfExpression, env *EvalEnv) Object {
	// For typeof, we want to determine the type without fully evaluating
	// This is a simplified implementation
	switch expr := node.Expr.(type) {
	case *IntegerLiteral:
		return &String{Value: string(IntType)}
	case *Identifier:
		// Try to get the identifier's value to determine its type
		val, ok := env.Get(expr.Value)
		if !ok {
			return newError("identifier not found: " + expr.Value)
		}
		return &String{Value: string(val.Type())}
	case *InfixExpression:
		// For expressions, we need to evaluate to determine type
		result := eval.Eval(expr, env)
		if isError(result) {
			return result
		}
		return &String{Value: string(result.Type())}
	case *PrefixExpression:
		result := eval.Eval(expr, env)
		if isError(result) {
			return result
		}
		return &String{Value: string(result.Type())}
	default:
		return newError("cannot determine type of expression: %T", expr)
	}
}

// Helper functions
func isError(obj Object) bool {
	if obj != nil {
		return obj.Type() == UnknownType && obj.Inspect()[:5] == "ERROR"
	}
	return false
}

func newError(format string, a ...interface{}) *Error {
	return &Error{Message: fmt.Sprintf(format, a...)}
}

func nativeBoolToPyMonkeyBoolean(input bool) Object {
	if input {
		return &Integer{Value: 1} // Using 1 for true
	}
	return &Integer{Value: 0} // Using 0 for false
}

// Helper function to convert string to object for testing
func StringToObject(s string) Object {
	if val, err := strconv.ParseInt(s, 10, 64); err == nil {
		return &Integer{Value: val}
	}
	return &String{Value: s}
}
