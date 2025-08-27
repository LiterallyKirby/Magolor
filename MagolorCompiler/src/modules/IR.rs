use crate::modules::parser::{AST, ASTValue};
use inkwell::AddressSpace;
use inkwell::context::Context;
use std::collections::HashMap;

use inkwell::values::PointerValue;
#[derive(Debug, Clone, Copy)]
pub enum VarType {
    Int,
    Str,
}

pub fn compile(ast: Vec<AST>) {
    // Create context, module, builder once
    let context = Context::create();
    let module = context.create_module("magolor");
    let builder = context.create_builder();

    // i32 type for main
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // Prepare C's puts function for console.print
    let i8_ptr = context.ptr_type(AddressSpace::from(0));
    let puts_type = i32_type.fn_type(&[i8_ptr.into()], false);
    let puts_fn = module.add_function("puts", puts_type, None);

    // symbol table

    let mut variables: HashMap<String, (PointerValue, VarType)> = HashMap::new();

    for node in ast {
        match node {
            AST::Call {
                object,
                method,
                args,
            } => {
                if object == "console" && method == "print" {
                    for arg in args {
                        match arg {
                            AST::VarRef(name) => {
                                if let Some((var_ptr, var_type)) = variables.get(&name) {
                                    let val = match var_type {
                                        VarType::Str => builder
                                            .build_load(i8_ptr, *var_ptr, &name)
                                            .expect("load failed"),
                                        VarType::Int => builder
                                            .build_load(i32_type, *var_ptr, &name)
                                            .expect("load failed"),
                                    };

                                    builder.build_call(puts_fn, &[val.into()], "call_puts");
                                } else {
                                    panic!("Unknown variable {}", name);
                                }
                            }

                            AST::VarDecl(_, _, ASTValue::Str(s)) => {
                                let c_string = builder
                                    .build_global_string_ptr(&s, "tmp")
                                    .expect("global str failed");
                                builder.build_call(
                                    puts_fn,
                                    &[c_string.as_pointer_value().into()],
                                    "call_puts",
                                );
                            }
                            AST::VarDecl(_, _, ASTValue::Int(n)) => {
                                let num_str = n.to_string();
                                let c_string = builder
                                    .build_global_string_ptr(&num_str, "tmp")
                                    .expect("global int str failed");
                                builder.build_call(
                                    puts_fn,
                                    &[c_string.as_pointer_value().into()],
                                    "call_puts",
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }

            AST::VarDecl(ty, name, value) => match value {
                ASTValue::Str(s) => {
                    println!("Declare var: {} {} = \"{}\"", ty, name, s);

                    let var_alloca = builder.build_alloca(i8_ptr, &name).expect("alloca failed");
                    let c_string = builder
                        .build_global_string_ptr(&s, &format!("{}_str", name))
                        .expect("global str failed");
                    builder.build_store(var_alloca, c_string.as_pointer_value());

                    // Insert with VarType::Str
                    variables.insert(name, (var_alloca, VarType::Str));
                }
                ASTValue::Int(n) => {
                    println!("Declare var: {} {} = {}", ty, name, n);

                    let var_alloca = builder
                        .build_alloca(i32_type, &name)
                        .expect("alloca failed");
                    let int_val = i32_type.const_int(n as u64, false);
                    builder.build_store(var_alloca, int_val);

                    // Insert with VarType::Int
                    variables.insert(name, (var_alloca, VarType::Int));
                }
            },

            AST::Import(pkg) => {
                println!("import: {}", pkg);
            }
            _ => {}
        }
    }

    // Return 0 at the end of main
    builder.build_return(Some(&i32_type.const_int(0, false)));

    // Print LLVM IR
    module.print_to_stderr();
}
