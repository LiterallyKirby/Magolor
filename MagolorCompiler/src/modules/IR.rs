use inkwell::context::Context;
use inkwell::AddressSpace;
use crate::modules::parser::AST;

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
    let i8_ptr = context.i8_type().ptr_type(AddressSpace::from(0));
    let puts_type = i8_ptr.fn_type(&[i8_ptr.into()], false);
    let puts_fn = module.add_function("puts", puts_type, None);

    for node in ast {
        match node {
            AST::Call { object, method, args } => {
                if object == "console" && method == "print" {
                    for arg in args {
                        match arg {
                            AST::VarRef(s) => {
                                let c_string = builder
                                    .build_global_string_ptr(&s, "tmp")
                                    .expect("Failed to create global string");
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
            AST::VarDecl(ty, name, value) => {
                println!("Declared variable: {} {} = {}", ty, name, value);
            }
            AST::Import(pkg) => {
                println!("Import: {}", pkg);
            }
            _ => {}
        }
    }

    // Return 0 at the end of main
    builder.build_return(Some(&i32_type.const_int(0, false)));

    // Print LLVM IR
    module.print_to_stderr();
}
