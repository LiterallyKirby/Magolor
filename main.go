package main

import "fmt"

func main() {
	// Test cases
	testCases :=  []string{
	`int fn main(int x, string y) { return 123 + 4 * 5; }`,
	`int func main(int x, string y){ return 123 + 4 * 5; }`,
	`int main(int x, string y) { return 123 + 4 * 5; }`,
	`void test() { return; }`,
	`float calculate(int a, float b, string name) { return a + b * -3; }`,
	`int test() { return typeof(42); }`,
	`void demo() { return typeof(x + y * 2); }`,
	`void testIf() { if (x > 10) { return x; } }`,
	`void testIfElse() { if (x > 10) { return x; } else { return 0; } }`,
	`void testElseIf() { if (x > 10) { return x; } else if (x > 5) { return x + 1; } else { return 0; } }`,
	`void testNoBraces() { if (x > 10) return x; else return 0; }`,
	`void testDeclaration() { int x = 10; return x; }`,
	`void testLoop() { loop { return 1; } }`,
	`void testWhile() { while (x < 10) { return x; } }`,
	`void testFor() { for (item in items) { return item; } }`,

	// Added string & float focused tests
	`string greet(string name) { return "Hello, " + name; }`,
	`float addFloats(float a, float b) { return a + b; }`,
	`string emptyString() { return ""; }`,
	`float negativeFloat() { return -3.14; }`,
	`void returnStringVoid() { return "not really void"; }`, // Should error or handle accordingly
}

	for i, source := range testCases {
		fmt.Printf("=== Test Case %d ===\n", i+1)
		fmt.Printf("Source: %s\n", source)

		lexer := NewLexer(source)
		parser := NewParser(lexer)

		program := parser.ParseProgram()

		if len(parser.errors) != 0 {
			fmt.Println("Parser errors:")
			for _, err := range parser.errors {
				fmt.Println(" -", err)
			}
		} else {
			fmt.Println("Parsed program:")
			fmt.Println(program.String())
		}
		fmt.Println()
	}

	// Demonstrate lexer functionality
	fmt.Println("=== Lexer Demo ===")
	demoLexer()

	// Demonstrate typeof evaluation
	fmt.Println("\n=== Typeof Evaluation Demo ===")
	demoTypeofEvaluation()
}

func demoLexer() {
	input := "int main(int x) { return x + 42; }"
	lexer := NewLexer(input)

	fmt.Printf("Tokenizing: %s\n", input)
	fmt.Println("Tokens:")

	for {
		tok := lexer.NextToken()
		fmt.Printf("  Type: %-10s Literal: %s\n", tok.Type, tok.Literal)
		if tok.Type == EOF {
			break
		}
	}
}

func demoTypeofEvaluation() {
	expressions := []string{
		"typeof(42)",
		"typeof(x + 5)",
		"typeof(-10)",
	}

	evaluator := NewEvaluator()
	env := NewEvalEnv()

	// Set up some variables in the environment
	env.Set("x", &Integer{Value: 100})
	env.Set("name", &String{Value: "hello"})

	for _, exprStr := range expressions {
		fmt.Printf("Evaluating: %s\n", exprStr)

		lexer := NewLexer(exprStr)
		parser := NewParser(lexer)

		// Parse as expression
		expr := parser.parseExpression(LOWEST)
		if len(parser.errors) != 0 {
			fmt.Println("  Parse errors:")
			for _, err := range parser.errors {
				fmt.Println("   -", err)
			}
			continue
		}

		result := evaluator.Eval(expr, env)
		fmt.Printf("  Result: %s (type: %s)\n", result.Inspect(), result.Type())
		fmt.Println()
	}
}
