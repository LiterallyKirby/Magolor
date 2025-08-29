use crate::modules::parser::{AST, ASTValue};
use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum VarType {
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    Str,
}

pub fn compile(ast: Vec<AST>) {
    // Create context, module, builder once
    let context = Context::create();
    let module = context.create_module("magolor");
    let builder = context.create_builder();

    // Basic types
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let bool_type = context.bool_type();

    let i8_ptr = context.ptr_type(AddressSpace::from(0));

    // Prepare C's puts function for console.print
    let puts_type = i32_type.fn_type(&[i8_ptr.into()], false);
    let puts_fn = module.add_function("puts", puts_type, None);

    // Global symbol table for functions
    let mut functions: HashMap<String, FunctionValue> = HashMap::new();

    // First pass: declare all functions
    for node in &ast {
        if let AST::FuncDef {
            name,
            params,
            return_type,
            ..
        } = node
        {
            let mut param_types = Vec::new();

            // Convert parameter types
            for (_, param_type) in params {
                let llvm_type = match param_type.as_str() {
                    "i32" => i32_type.into(),
                    "i64" => i64_type.into(),
                    "f32" => f32_type.into(),
                    "f64" => f64_type.into(),
                    "bool" => bool_type.into(),
                    "str" => i8_ptr.into(),
                    _ => panic!("Unsupported parameter type: {}", param_type),
                };
                param_types.push(llvm_type);
            }

            // Determine return type
            let ret_type = match return_type.as_deref() {
                Some("void") | None => i32_type.fn_type(&param_types, false),
                Some("i32") => i32_type.fn_type(&param_types, false),
                Some("i64") => i64_type.fn_type(&param_types, false),
                Some("f32") => f32_type.fn_type(&param_types, false),
                Some("f64") => f64_type.fn_type(&param_types, false),
                Some("bool") => bool_type.fn_type(&param_types, false),
                Some("str") => i8_ptr.fn_type(&param_types, false),
                Some(rt) => panic!("Unsupported return type: {}", rt),
            };

            let function = module.add_function(name, ret_type, None);
            functions.insert(name.clone(), function);
        }
    }

    // Second pass: compile function bodies
    for node in ast {
        match node {
            AST::FuncDef {
                name, params, body, ..
            } => {
                let function = functions.get(&name).unwrap();
                let basic_block = context.append_basic_block(*function, "entry");
                builder.position_at_end(basic_block);

                // Local symbol table for this function
                let mut variables: HashMap<String, (PointerValue, VarType)> = HashMap::new();

                // Add parameters to symbol table
                for (i, (param_name, param_type)) in params.iter().enumerate() {
                    let param_value = function.get_nth_param(i as u32).unwrap();
                    let var_type = match param_type.as_str() {
                        "i32" => VarType::Int32,
                        "i64" => VarType::Int64,
                        "f32" => VarType::Float32,
                        "f64" => VarType::Float64,
                        "bool" => VarType::Bool,
                        "str" => VarType::Str,
                        _ => panic!("Unsupported parameter type: {}", param_type),
                    };

                    // Allocate space for parameter and store it
                    let alloca = match var_type {
                        VarType::Int32 => builder
                            .build_alloca(i32_type, param_name)
                            .expect("alloca failed"),
                        VarType::Int64 => builder
                            .build_alloca(i64_type, param_name)
                            .expect("alloca failed"),
                        VarType::Float32 => builder
                            .build_alloca(f32_type, param_name)
                            .expect("alloca failed"),
                        VarType::Float64 => builder
                            .build_alloca(f64_type, param_name)
                            .expect("alloca failed"),
                        VarType::Bool => builder
                            .build_alloca(bool_type, param_name)
                            .expect("alloca failed"),
                        VarType::Str => builder
                            .build_alloca(i8_ptr, param_name)
                            .expect("alloca failed"),
                    };
                    builder.build_store(alloca, param_value);
                    variables.insert(param_name.clone(), (alloca, var_type));
                }

                // Compile function body
                compile_statements(
                    &body,
                    &builder,
                    &context,
                    &puts_fn,
                    &mut variables,
                    &functions,
                );

                // Add return if not present
                let last_block = builder.get_insert_block().unwrap();
                if last_block.get_terminator().is_none() {
                    builder.build_return(Some(&i32_type.const_int(0, false)));
                }
            }

            AST::Import(pkg) => {
                println!("import: {}", pkg);
            }

            // Handle top-level statements (if any)
            _ => {
                // For top-level code, we'd need a main function
                // This is a simplified approach - you might want to handle this differently
                println!("Warning: Top-level statement found, skipping: {:?}", node);
            }
        }
    }

    // Print LLVM IR
    module.print_to_stderr();
}

fn compile_statements<'ctx>(
    statements: &[AST],
    builder: &inkwell::builder::Builder<'ctx>,
    context: &'ctx Context,
    puts_fn: &FunctionValue,
    variables: &mut HashMap<String, (PointerValue<'ctx>, VarType)>,
    _functions: &HashMap<String, FunctionValue>,
) {
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let bool_type = context.bool_type();
    let i8_ptr = context.ptr_type(AddressSpace::from(0));

    for stmt in statements {
        match stmt {
            AST::Call {
                object,
                method,
                args,
            } => {
                if object == "console" && method == "print" {
                    for arg in args {
                        match arg {
                            AST::VarRef(name) => {
                                if let Some((var_ptr, var_type)) = variables.get(name) {
                                    let val = match var_type {
                                        VarType::Str => builder
                                            .build_load(i8_ptr, *var_ptr, name)
                                            .expect("load failed"),
                                        VarType::Int32 => {
                                            let int_val = builder
                                                .build_load(i32_type, *var_ptr, name)
                                                .expect("load failed");
                                            // Convert to string for printing
                                            let num_str = format!("%d"); // Using format string for printf-style
                                            let c_string = builder
                                                .build_global_string_ptr(&num_str, "tmp")
                                                .expect("global str failed");
                                            c_string.as_pointer_value().into()
                                        },
                                        VarType::Int64 => {
                                            let int_val = builder
                                                .build_load(i64_type, *var_ptr, name)
                                                .expect("load failed");
                                            let num_str = format!("%lld"); // Format for long long
                                            let c_string = builder
                                                .build_global_string_ptr(&num_str, "tmp")
                                                .expect("global str failed");
                                            c_string.as_pointer_value().into()
                                        },
                                        VarType::Float32 => {
                                            let float_val = builder
                                                .build_load(f32_type, *var_ptr, name)
                                                .expect("load failed");
                                            let num_str = format!("%.2f"); // Format for float
                                            let c_string = builder
                                                .build_global_string_ptr(&num_str, "tmp")
                                                .expect("global str failed");
                                            c_string.as_pointer_value().into()
                                        },
                                        VarType::Float64 => {
                                            let float_val = builder
                                                .build_load(f64_type, *var_ptr, name)
                                                .expect("load failed");
                                            let num_str = format!("%.2lf"); // Format for double
                                            let c_string = builder
                                                .build_global_string_ptr(&num_str, "tmp")
                                                .expect("global str failed");
                                            c_string.as_pointer_value().into()
                                        },
                                        VarType::Bool => {
                                            let bool_val = builder
                                                .build_load(bool_type, *var_ptr, name)
                                                .expect("load failed");
                                            // Convert bool to "true" or "false" string
                                            let true_str = builder
                                                .build_global_string_ptr("true", "true_str")
                                                .expect("global str failed");
                                            let false_str = builder
                                                .build_global_string_ptr("false", "false_str")
                                                .expect("global str failed");
                                            
                                            let selected = builder.build_select(
                                                bool_val.into_int_value(),
                                                true_str.as_pointer_value(),
                                                false_str.as_pointer_value(),
                                                "bool_str_select"
                                            ).expect("select failed");
                                            selected.into()
                                        },
                                    };

                                    builder.build_call(*puts_fn, &[val.into()], "call_puts");
                                } else {
                                    panic!("Unknown variable {}", name);
                                }
                            }

                            AST::Literal(ASTValue::Str(s)) => {
                                let c_string = builder
                                    .build_global_string_ptr(s, "tmp")
                                    .expect("global str failed");
                                builder.build_call(
                                    *puts_fn,
                                    &[c_string.as_pointer_value().into()],
                                    "call_puts",
                                );
                            }
                            AST::Literal(ASTValue::Int(n)) => {
                                let num_str = n.to_string();
                                let c_string = builder
                                    .build_global_string_ptr(&num_str, "tmp")
                                    .expect("global int str failed");
                                builder.build_call(
                                    *puts_fn,
                                    &[c_string.as_pointer_value().into()],
                                    "call_puts",
                                );
                            }
                            AST::Literal(ASTValue::Float32(f)) => {
                                let num_str = f.to_string();
                                let c_string = builder
                                    .build_global_string_ptr(&num_str, "tmp")
                                    .expect("global float str failed");
                                builder.build_call(
                                    *puts_fn,
                                    &[c_string.as_pointer_value().into()],
                                    "call_puts",
                                );
                            }
                            AST::Literal(ASTValue::Float64(f)) => {
                                let num_str = f.to_string();
                                let c_string = builder
                                    .build_global_string_ptr(&num_str, "tmp")
                                    .expect("global float str failed");
                                builder.build_call(
                                    *puts_fn,
                                    &[c_string.as_pointer_value().into()],
                                    "call_puts",
                                );
                            }
                            AST::Literal(ASTValue::Int64(n)) => {
                                let num_str = n.to_string();
                                let c_string = builder
                                    .build_global_string_ptr(&num_str, "tmp")
                                    .expect("global int64 str failed");
                                builder.build_call(
                                    *puts_fn,
                                    &[c_string.as_pointer_value().into()],
                                    "call_puts",
                                );
                            }
                            AST::Literal(ASTValue::Bool(b)) => {
                                let bool_str = if *b { "true" } else { "false" };
                                let c_string = builder
                                    .build_global_string_ptr(bool_str, "tmp")
                                    .expect("global bool str failed");
                                builder.build_call(
                                    *puts_fn,
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

                    let var_alloca = builder.build_alloca(i8_ptr, name).expect("alloca failed");
                    let c_string = builder
                        .build_global_string_ptr(s, &format!("{}_str", name))
                        .expect("global str failed");
                    builder.build_store(var_alloca, c_string.as_pointer_value());

                    variables.insert(name.clone(), (var_alloca, VarType::Str));
                }
                ASTValue::Int(n) => {
                    println!("Declare var: {} {} = {}", ty, name, n);

                    // Determine the correct integer type based on the declared type
                    let (var_alloca, var_type) = match ty.as_str() {
                        "i32" => {
                            let alloca = builder.build_alloca(i32_type, name).expect("alloca failed");
                            let int_val = i32_type.const_int(*n as u64, false);
                            builder.build_store(alloca, int_val);
                            (alloca, VarType::Int32)
                        }
                        "i64" => {
                            let alloca = builder.build_alloca(i64_type, name).expect("alloca failed");
                            let int_val = i64_type.const_int(*n as u64, false);
                            builder.build_store(alloca, int_val);
                            (alloca, VarType::Int64)
                        }
                        _ => {
                            // Default to i32 for backward compatibility
                            let alloca = builder.build_alloca(i32_type, name).expect("alloca failed");
                            let int_val = i32_type.const_int(*n as u64, false);
                            builder.build_store(alloca, int_val);
                            (alloca, VarType::Int32)
                        }
                    };

                    variables.insert(name.clone(), (var_alloca, var_type));
                }
                ASTValue::Int64(n) => {
                    println!("Declare var: {} {} = {}", ty, name, n);

                    // Determine the correct integer type based on the declared type
                    let (var_alloca, var_type) = match ty.as_str() {
                        "i32" => {
                            let alloca = builder.build_alloca(i32_type, name).expect("alloca failed");
                            let int_val = i32_type.const_int(*n as u64, false);
                            builder.build_store(alloca, int_val);
                            (alloca, VarType::Int32)
                        }
                        "i64" => {
                            let alloca = builder.build_alloca(i64_type, name).expect("alloca failed");
                            let int_val = i64_type.const_int(*n as u64, false);
                            builder.build_store(alloca, int_val);
                            (alloca, VarType::Int64)
                        }
                        _ => {
                            // Default to i64 for Int64 literals
                            let alloca = builder.build_alloca(i64_type, name).expect("alloca failed");
                            let int_val = i64_type.const_int(*n as u64, false);
                            builder.build_store(alloca, int_val);
                            (alloca, VarType::Int64)
                        }
                    };

                    variables.insert(name.clone(), (var_alloca, var_type));
                }
                ASTValue::Float32(f) => {
                    println!("Declare var: {} {} = {}", ty, name, f);

                    // Determine the correct float type based on the declared type
                    let (var_alloca, var_type) = match ty.as_str() {
                        "f32" => {
                            let alloca = builder.build_alloca(f32_type, name).expect("alloca failed");
                            let float_val = f32_type.const_float(*f as f64);
                            builder.build_store(alloca, float_val);
                            (alloca, VarType::Float32)
                        }
                        "f64" => {
                            let alloca = builder.build_alloca(f64_type, name).expect("alloca failed");
                            let float_val = f64_type.const_float(*f as f64);
                            builder.build_store(alloca, float_val);
                            (alloca, VarType::Float64)
                        }
                        _ => {
                            // Default to f32 for Float32 literals
                            let alloca = builder.build_alloca(f32_type, name).expect("alloca failed");
                            let float_val = f32_type.const_float(*f as f64);
                            builder.build_store(alloca, float_val);
                            (alloca, VarType::Float32)
                        }
                    };

                    variables.insert(name.clone(), (var_alloca, var_type));
                }
                ASTValue::Float64(f) => {
                    println!("Declare var: {} {} = {}", ty, name, f);

                    // Determine the correct float type based on the declared type
                    let (var_alloca, var_type) = match ty.as_str() {
                        "f32" => {
                            let alloca = builder.build_alloca(f32_type, name).expect("alloca failed");
                            let float_val = f32_type.const_float(*f);
                            builder.build_store(alloca, float_val);
                            (alloca, VarType::Float32)
                        }
                        "f64" => {
                            let alloca = builder.build_alloca(f64_type, name).expect("alloca failed");
                            let float_val = f64_type.const_float(*f);
                            builder.build_store(alloca, float_val);
                            (alloca, VarType::Float64)
                        }
                        _ => {
                            // Default to f64 for Float64 literals
                            let alloca = builder.build_alloca(f64_type, name).expect("alloca failed");
                            let float_val = f64_type.const_float(*f);
                            builder.build_store(alloca, float_val);
                            (alloca, VarType::Float64)
                        }
                    };

                    variables.insert(name.clone(), (var_alloca, var_type));
                }
                ASTValue::Bool(b) => {
                    println!("Declare var: {} {} = {}", ty, name, b);

                    let var_alloca = builder.build_alloca(bool_type, name).expect("alloca failed");
                    let bool_val = bool_type.const_int(if *b { 1 } else { 0 }, false);
                    builder.build_store(var_alloca, bool_val);

                    variables.insert(name.clone(), (var_alloca, VarType::Bool));
                }
                ASTValue::VarRef(ref_name) => {
                    // Handle variable assignment from another variable
                    if let Some((src_ptr, src_type)) = variables.get(ref_name) {
                        let var_alloca = match src_type {
                            VarType::Int32 => {
                                builder.build_alloca(i32_type, name).expect("alloca failed")
                            }
                            VarType::Int64 => {
                                builder.build_alloca(i64_type, name).expect("alloca failed")
                            }
                            VarType::Float32 => {
                                builder.build_alloca(f32_type, name).expect("alloca failed")
                            }
                            VarType::Float64 => {
                                builder.build_alloca(f64_type, name).expect("alloca failed")
                            }
                            VarType::Bool => {
                                builder.build_alloca(bool_type, name).expect("alloca failed")
                            }
                            VarType::Str => {
                                builder.build_alloca(i8_ptr, name).expect("alloca failed")
                            }
                        };

                        let src_val = match src_type {
                            VarType::Int32 => builder
                                .build_load(i32_type, *src_ptr, ref_name)
                                .expect("load failed"),
                            VarType::Int64 => builder
                                .build_load(i64_type, *src_ptr, ref_name)
                                .expect("load failed"),
                            VarType::Float32 => builder
                                .build_load(f32_type, *src_ptr, ref_name)
                                .expect("load failed"),
                            VarType::Float64 => builder
                                .build_load(f64_type, *src_ptr, ref_name)
                                .expect("load failed"),
                            VarType::Bool => builder
                                .build_load(bool_type, *src_ptr, ref_name)
                                .expect("load failed"),
                            VarType::Str => builder
                                .build_load(i8_ptr, *src_ptr, ref_name)
                                .expect("load failed"),
                        };

                        builder.build_store(var_alloca, src_val);
                        variables.insert(name.clone(), (var_alloca, *src_type));
                    } else {
                        panic!("Unknown variable reference: {}", ref_name);
                    }
                }
                ASTValue::FuncCall {
                    name: func_name,
                    args,
                } => {
                    // Handle function call assignment
                    if let Some(function) = _functions.get(func_name) {
                        // Prepare arguments for function call
                        let mut call_args = Vec::new();

                        for arg in args {
                            match arg {
                                ASTValue::Int(n) => {
                                    call_args.push(i32_type.const_int(*n as u64, false).into());
                                }
                                ASTValue::Int64(n) => {
                                    call_args.push(i64_type.const_int(*n as u64, false).into());
                                }
                                ASTValue::Float32(f) => {
                                    call_args.push(f32_type.const_float(*f as f64).into());
                                }
                                ASTValue::Float64(f) => {
                                    call_args.push(f64_type.const_float(*f).into());
                                }
                                ASTValue::Bool(b) => {
                                    call_args.push(bool_type.const_int(if *b { 1 } else { 0 }, false).into());
                                }
                                ASTValue::Str(s) => {
                                    let c_string = builder
                                        .build_global_string_ptr(s, "arg_str")
                                        .expect("global str failed");
                                    call_args.push(c_string.as_pointer_value().into());
                                }
                                ASTValue::VarRef(var_name) => {
                                    if let Some((var_ptr, var_type)) = variables.get(var_name) {
                                        let val = match var_type {
                                            VarType::Int32 => builder
                                                .build_load(i32_type, *var_ptr, var_name)
                                                .expect("load failed"),
                                            VarType::Int64 => builder
                                                .build_load(i64_type, *var_ptr, var_name)
                                                .expect("load failed"),
                                            VarType::Float32 => builder
                                                .build_load(f32_type, *var_ptr, var_name)
                                                .expect("load failed"),
                                            VarType::Float64 => builder
                                                .build_load(f64_type, *var_ptr, var_name)
                                                .expect("load failed"),
                                            VarType::Bool => builder
                                                .build_load(bool_type, *var_ptr, var_name)
                                                .expect("load failed"),
                                            VarType::Str => builder
                                                .build_load(i8_ptr, *var_ptr, var_name)
                                                .expect("load failed"),
                                        };
                                        call_args.push(val.into());
                                    } else {
                                        panic!("Unknown variable in function call: {}", var_name);
                                    }
                                }
                                ASTValue::FuncCall { .. } => {
                                    // Nested function calls - you'd need to implement this recursively
                                    panic!("Nested function calls not yet implemented");
                                }
                            }
                        }

                        // Make the function call
                        let call_result = builder
                            .build_call(*function, &call_args, "func_call")
                            .expect("function call failed");

                        // Store the result in a new variable
                        // For now, assuming functions return i32 (you might need to track return types)
                        let var_alloca =
                            builder.build_alloca(i32_type, name).expect("alloca failed");
                        if let Some(result_value) = call_result.try_as_basic_value().left() {
                            builder.build_store(var_alloca, result_value);
                            variables.insert(name.clone(), (var_alloca, VarType::Int32));
                        } else {
                            // Function returned void, store a default value
                            let default_val = i32_type.const_int(0, false);
                            builder.build_store(var_alloca, default_val);
                            variables.insert(name.clone(), (var_alloca, VarType::Int32));
                        }
                    } else {
                        panic!("Unknown function: {}", func_name);
                    }
                }
            },

          AST::GreaterThan =>{

            }

            AST::Return(value) => {
                match value {
                    ASTValue::Int(n) => {
                        let ret_val = i32_type.const_int(*n as u64, false);
                        builder.build_return(Some(&ret_val));
                    }
                    ASTValue::Int64(n) => {
                        let ret_val = i64_type.const_int(*n as u64, false);
                        builder.build_return(Some(&ret_val));
                    }
                    ASTValue::Float32(f) => {
                        let ret_val = f32_type.const_float(*f as f64);
                        builder.build_return(Some(&ret_val));
                    }
                    ASTValue::Float64(f) => {
                        let ret_val = f64_type.const_float(*f);
                        builder.build_return(Some(&ret_val));
                    }
                    ASTValue::Bool(b) => {
                        let ret_val = bool_type.const_int(if *b { 1 } else { 0 }, false);
                        builder.build_return(Some(&ret_val));
                    }
                    ASTValue::Str(_) => {
                        // For string returns, you'd need to handle this based on your ABI
                        // This is a simplified version
                        let ret_val = i32_type.const_int(0, false);
                        builder.build_return(Some(&ret_val));
                    }
                    ASTValue::VarRef(name) => {
                        if let Some((var_ptr, var_type)) = variables.get(name) {
                            match var_type {
                                VarType::Int32 => {
                                    let val = builder
                                        .build_load(i32_type, *var_ptr, name)
                                        .expect("load failed");
                                    builder.build_return(Some(&val.into_int_value()));
                                }
                                VarType::Int64 => {
                                    let val = builder
                                        .build_load(i64_type, *var_ptr, name)
                                        .expect("load failed");
                                    builder.build_return(Some(&val.into_int_value()));
                                }
                                VarType::Float32 => {
                                    let val = builder
                                        .build_load(f32_type, *var_ptr, name)
                                        .expect("load failed");
                                    builder.build_return(Some(&val.into_float_value()));
                                }
                                VarType::Float64 => {
                                    let val = builder
                                        .build_load(f64_type, *var_ptr, name)
                                        .expect("load failed");
                                    builder.build_return(Some(&val.into_float_value()));
                                }
                                VarType::Bool => {
                                    let val = builder
                                        .build_load(bool_type, *var_ptr, name)
                                        .expect("load failed");
                                    builder.build_return(Some(&val.into_int_value()));
                                }
                                VarType::Str => {
                                    // Again, simplified - string return handling depends on your ABI
                                    let ret_val = i32_type.const_int(0, false);
                                    builder.build_return(Some(&ret_val));
                                }
                            }
                        } else {
                            panic!("Unknown variable in return: {}", name);
                        }
                    }
                    ASTValue::FuncCall {
                        name: func_name,
                        args,
                    } => {
                        // Handle function call in return statement
                        if let Some(function) = _functions.get(func_name) {
                            let mut call_args = Vec::new();

                            for arg in args {
                                match arg {
                                    ASTValue::Int(n) => {
                                        call_args.push(i32_type.const_int(*n as u64, false).into());
                                    }
                                    ASTValue::Int64(n) => {
                                        call_args.push(i64_type.const_int(*n as u64, false).into());
                                    }
                                    ASTValue::Float32(f) => {
                                        call_args.push(f32_type.const_float(*f as f64).into());
                                    }
                                    ASTValue::Float64(f) => {
                                        call_args.push(f64_type.const_float(*f).into());
                                    }
                                    ASTValue::Bool(b) => {
                                        call_args.push(bool_type.const_int(if *b { 1 } else { 0 }, false).into());
                                    }
                                    ASTValue::Str(s) => {
                                        let c_string = builder
                                            .build_global_string_ptr(s, "ret_arg_str")
                                            .expect("global str failed");
                                        call_args.push(c_string.as_pointer_value().into());
                                    }
                                    ASTValue::VarRef(var_name) => {
                                        if let Some((var_ptr, var_type)) = variables.get(var_name) {
                                            let val = match var_type {
                                                VarType::Int32 => builder
                                                    .build_load(i32_type, *var_ptr, var_name)
                                                    .expect("load failed"),
                                                VarType::Int64 => builder
                                                    .build_load(i64_type, *var_ptr, var_name)
                                                    .expect("load failed"),
                                                VarType::Float32 => builder
                                                    .build_load(f32_type, *var_ptr, var_name)
                                                    .expect("load failed"),
                                                VarType::Float64 => builder
                                                    .build_load(f64_type, *var_ptr, var_name)
                                                    .expect("load failed"),
                                                VarType::Bool => builder
                                                    .build_load(bool_type, *var_ptr, var_name)
                                                    .expect("load failed"),
                                                VarType::Str => builder
                                                    .build_load(i8_ptr, *var_ptr, var_name)
                                                    .expect("load failed"),
                                            };
                                            call_args.push(val.into());
                                        } else {
                                            panic!(
                                                "Unknown variable in function call: {}",
                                                var_name
                                            );
                                        }
                                    }
                                    ASTValue::FuncCall { .. } => {
                                        panic!(
                                            "Nested function calls in return not yet implemented"
                                        );
                                    }
                                }
                            }

                            let call_result = builder
                                .build_call(*function, &call_args, "ret_func_call")
                                .expect("function call failed");
                            if let Some(result_value) = call_result.try_as_basic_value().left() {
                                // The into_*_value() methods don't return Result, they panic on wrong type
                                // So we need to handle this differently
                                builder.build_return(Some(&result_value.into_int_value()));
                            } else {
                                let ret_val = i32_type.const_int(0, false);
                                builder.build_return(Some(&ret_val));
                            }
                        } else {
                            panic!("Unknown function in return: {}", func_name);
                        }
                    }
                }
                return; // Exit early since we've returned
            }

            AST::NewLine => {
                // Skip newlines
            }

            _ => {
                println!("Unhandled statement: {:?}", stmt);
            }
        }
    }
}
