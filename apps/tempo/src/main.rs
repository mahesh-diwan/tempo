fn main() {
    if std::env::args().any(|a| a == "--version") {
        println!("tempo {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    println!("tempo");
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_zero_one() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
    }
}
