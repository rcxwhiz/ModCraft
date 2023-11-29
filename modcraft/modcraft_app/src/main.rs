use modcraft_lib::add;

fn main() {
    println!("Welcome to modcraft");

    #[cfg(feature = "client")]
    println!("Compiled as client");

    #[cfg(not(feature = "client"))]
    println!("Compiled as stand alone server");

    println!("Modcraft lib is working: {} + {} = {}", 2, 2, add(2, 2));
}
