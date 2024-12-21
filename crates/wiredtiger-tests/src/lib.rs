extern crate wiredtiger;

#[cfg(test)]
mod tests {
    //use wiredtiger::wiredtiger_format;

    //#[wiredtiger_format]
    struct MyKeyType {}

    #[test]
    fn testit() {
        let _ = MyKeyType {};
    }

    fn test_basic() {}
}
