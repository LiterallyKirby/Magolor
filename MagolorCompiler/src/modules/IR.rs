
use inkwell::context::Context;

pub fn compile() {
    let context = Context::create();
    let module = context.create_module("magolor");
    let builder = context.create_builder();

    // i32 type
    let i32_type = context.i32_type();

    // function: int main()
    let fn_type = i32_type.fn_type(&[], false);
    let function = module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // const ints
    let x = i32_type.const_int(5, false);
    let y = i32_type.const_int(10, false);

    // add x + y
    let sum = builder.build_int_add(x, y, "sum").unwrap(); // no `?` because build_int_add returns IntValue directly
    builder.build_return(Some(&sum));

    module.print_to_stderr();
}

