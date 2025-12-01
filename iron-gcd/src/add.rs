#[no_mangle] // WebAssembly test
pub extern "C" fn add(x: i32, y: i32) -> i32 {
    x + y
}