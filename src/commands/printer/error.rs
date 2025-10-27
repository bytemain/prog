pub fn handle_inquire_error(e: inquire::InquireError) {
    eprintln!();
    match e {
        inquire::InquireError::OperationCanceled => {
            eprintln!("operation canceled");
        }
        inquire::InquireError::OperationInterrupted => {
            eprintln!("operation interrupted");
        }
        _ => {
            eprintln!("unknown error: {}", e);
        }
    }
}
