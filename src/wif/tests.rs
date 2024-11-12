#[test]
fn test() {
    let s = include_str!("../sample.wif");
    super::parse(s);
}
