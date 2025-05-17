pub fn handle_inquire_error(e: inquire::InquireError) {
    println!();
    match e {
        inquire::InquireError::OperationCanceled => {
            println!("operation canceled");
        }
        inquire::InquireError::OperationInterrupted => {
            println!("operation interrupted");
        }
        _ => {
            println!("unknown error: {}", e);
        }
    }
}
