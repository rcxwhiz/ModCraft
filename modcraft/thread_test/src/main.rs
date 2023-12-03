use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

fn main() {
    // Wrap the boolean flag in a Mutex
    let exit_flag = Arc::new(Mutex::new(false));

    // Clone the Arc for the child thread
    let child_exit_flag = Arc::clone(&exit_flag);

    // Spawn the child thread
    let handle = thread::spawn(move || {
        // Child thread regularly checks the flag
        while !*child_exit_flag.lock().unwrap() {
            // Your child thread logic here
            println!("Child thread working...");
            thread::sleep(Duration::from_secs(1));
        }

        println!("Child thread exiting...");
    });

    // Let the child thread run for a while...
    thread::sleep(Duration::from_secs(10));

    // Parent thread updates the flag by acquiring a lock
    *exit_flag.lock().unwrap() = true;

    // Wait for the child thread to finish
    handle.join().unwrap();

    println!("Parent thread exiting...");
}
