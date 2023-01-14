extern crate core;

use crate::env::VMEnv;
use crate::interpret::Interpreter;
use crate::vm::VM;

pub mod vm;
pub mod env;
pub mod interpret;
pub mod loader;

#[test]
fn test_basic_math() {
    use vm::VM;
    use env::VMEnv;
    let vm = VM::new(64);
    let mut env = VMEnv::of(vm, Interpreter::new());
    env.iconst(8);
    env.iconst(2);
    env.iadd();
    env.iconst(8);
    env.iadd();
    env.print();
}

#[test]
fn test_basic_class_load() {
    use loader::Loader;
    let loader = Loader{};
    println!("{:?}", loader.load_from_file("/tmp/x/Test.class").unwrap());
}

#[test]
fn test_hard_class_load() {
    use loader::Loader;
    let loader = Loader{};
    println!("{:?}", loader.load_from_file("/tmp/x/More.class").unwrap());
}

#[test]
fn test_very_hard_class_load() {
    use loader::Loader;
    let loader = Loader{};
    println!("{:?}", loader.load_from_file("/tmp/x/Fields.class").unwrap());
}

#[test]
fn test_impossibly_hard_class_load() {
    use loader::Loader;
    let loader = Loader{};
    println!("{:?}", loader.load_from_file("/Users/user/IdeaProjects/cleaner/build/classes/kotlin/main/Options.class").unwrap());
}

#[test]
fn test_impossibly_hard_class_load_with_interpret() {
    use loader::Loader;
    let loader = Loader{};
    let class = loader.load_from_file("/Users/user/IdeaProjects/cleaner/build/classes/kotlin/main/Options.class").unwrap();
    let main = class.get_main().unwrap();
    let mut env = VMEnv::of(VM::new(1024), Interpreter::new());
    env.execute(&main.code).expect("executed");
}


fn main() {
}
