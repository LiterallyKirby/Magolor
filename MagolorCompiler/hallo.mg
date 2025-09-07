use Console;

void fn main() {
    console.print("=== TEST START ===");


    let i32 a = 10;
    let i64 b = 20;
    let i32 c = 30;


    let i32 f_res = foo();


    if (a < b) {
        console.print("a < b works");
    } else {
        console.print("a < b fails");
    }

    if (b > c) {
        console.print("b > c works");
    } else {
        console.print("b > c fails");
    }

    if (a <= c) {
        console.print("a <= c works");
    } else {
        console.print("a <= c fails");
    }

    if (b >= a) {
        console.print("b >= a works");
    } else {
        console.print("b >= a fails");
    }

    if (a == 10) {
        console.print("a == 10 works");
    } else {
        console.print("a == 10 fails");
    }

    if (b != c) {
        console.print("b != c works");
    } else {
        console.print("b != c fails");
    }


    if (foo() > 50) {
        console.print("foo() > 50 works");
    }

    console.print("=== TEST END ===");
}


i32 fn foo() {
    console.print("Inside foo()");
    let i32 x = 42;
    console.print(x);
    return x;  // last statement
}

