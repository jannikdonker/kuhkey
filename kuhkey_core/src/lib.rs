pub fn hello_message() -> String {
    "Hello from core!".into()
}

pub fn say_hello() {
    println!("{}", hello_message())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_message_is_correct() {
        assert_eq!(hello_message(), "Hello from core!");
    }
}
