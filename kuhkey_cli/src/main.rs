fn main() {
    say_hello();
}

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

    #[test]
    fn prints_same_as_core() {
        assert_eq!(hello_message(), kuhkey_core::hello_message());
    }
}
