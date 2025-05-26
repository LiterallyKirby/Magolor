package main

// Type represents different types in the language
type Type string

const (
	IntType     Type = "int"
	StringType  Type = "string"
	VoidType    Type = "void"
	FloatType   Type = "float"
	UnknownType Type = "unknown" // fallback
)

// Env represents the type environment for type checking
type Env struct {
	types map[string]Type
}

// NewEnv creates a new type environment
func NewEnv() *Env {
	return &Env{types: make(map[string]Type)}
}

// Get retrieves a type for a given name
func (e *Env) Get(name string) Type {
	if t, ok := e.types[name]; ok {
		return t
	}
	return UnknownType
}

// Set stores a type for a given name
func (e *Env) Set(name string, t Type) {
	e.types[name] = t
}
