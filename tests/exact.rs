//! Exact values: parsing, refusal, correctly rounded boundary conversion,
//! exact losses, checked arithmetic, text preservation.

use exactbench::exact::Exact;
use std::cmp::Ordering;

fn ex(s: &str) -> Exact { Exact::parse(s).unwrap() }

#[test]
fn recorded_forms_parse_to_reduced_ratios_and_keep_their_text() {
    for (text, num, den) in [("0.1235", 247, 2000), ("247/2000", 247, 2000), ("-12", -12, 1), ("-0.0035", -7, 2000),
                             ("0", 0, 1), ("-0", 0, 1), ("0.2000", 1, 5), ("00.50", 1, 2)] {
        let e = ex(text);
        assert_eq!((e.numer(), e.denom(), e.text()), (num, den, text), "{text}");
    }
    assert_eq!(ex("0.5"), ex("1/2"));
    assert_eq!(serde_json::to_string(&ex("0.2000")).unwrap(), "\"0.2000\"");
    assert_eq!(serde_json::from_str::<Exact>("\"0.2000\"").unwrap().text(), "0.2000");
}

#[test]
fn malformed_or_unrepresentable_text_is_refused() {
    for text in ["", "-", "+1", " 1", "1 ", "1e-3", "1.", ".5", "1/0", "6/-3", "1/", "/2", "abc", "0x10", "1.2.3", "1/2/3", "1.5/2",
                 "170141183460469231731687303715884105728", "0.000000000000000000000000000000000000001"] {
        assert!(Exact::parse(text).is_err(), "'{text}' accepted");
    }
    assert!(serde_json::from_str::<Exact>("0.5").is_err(), "a JSON number is not an exact value");
}

#[test]
fn boundary_conversion_is_correctly_rounded() {
    for text in ["0.1235", "0.2000", "-0.0035", "0.1", "0.30000000000000004", "9007199254740993", "9007199254740995",
                 "-9007199254740993", "1234567.000000001", "0.025500000000000002"] {
        assert_eq!(ex(text).to_f64().to_bits(), text.parse::<f64>().unwrap().to_bits(), "{text}");
    }
    assert_eq!(ex("123456789012345678901234567/1000").to_f64(), "123456789012345678901234.567".parse::<f64>().unwrap());
    // Deterministic sweep: ratios with exact binary64 operands (one IEEE division is correctly
    // rounded) and decimals of up to 38 digits (Rust's parser is correctly rounded).
    let mut s: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = || { s ^= s << 13; s ^= s >> 7; s ^= s << 17; s };
    for _ in 0..20_000 {
        let (n, d) = ((next() % (1 << 53)) as i128 - (1 << 52), (next() % (1 << 53)) as i128 + 1);
        let e = Exact::from_ratio(n, d).unwrap();
        assert_eq!(e.to_f64_uncached().to_bits(), (n as f64 / d as f64).to_bits(), "{n}/{d}");
        let digits = format!("{}{}", next(), next());
        let int = &digits[..(1 + next() % 38) as usize % digits.len().max(1)];
        let int = if int.is_empty() { "7" } else { int };
        let point = (next() % 38) as usize;
        let text = if point == 0 || point >= int.len() { int.to_string() } else { format!("{}.{}", &int[..int.len() - point], &int[int.len() - point..]) };
        assert_eq!(ex(&text).to_f64().to_bits(), text.parse::<f64>().unwrap().to_bits(), "{text}");
    }
}

#[test]
fn cached_and_uncached_conversion_agree_and_copies_keep_the_value() {
    let e = ex("0.025500000000000002");
    let (cached, uncached) = (e.to_f64(), e.to_f64_uncached());
    assert_eq!(cached.to_bits(), uncached.to_bits());
    let (shared, owned) = (e.clone(), e.deep_copy());
    assert_eq!((shared.to_f64().to_bits(), owned.to_f64().to_bits()), (cached.to_bits(), cached.to_bits()));
    assert_eq!((shared.text(), owned.text()), (e.text(), e.text()));
}

#[test]
fn binary64_values_convert_exactly_and_losses_are_exact() {
    let tenth = Exact::from_f64(0.1).unwrap();
    assert_eq!((tenth.numer(), tenth.denom()), (3602879701896397, 36028797018963968));
    assert!(ex("0.5").f64_loss().unwrap().is_zero());
    let loss = ex("0.1").f64_loss().unwrap();
    assert_eq!(loss, ex("1/10").checked_sub(&tenth).unwrap());
    assert_eq!(loss.signum(), -1);
    assert!(Exact::from_f64(f64::NAN).is_none() && Exact::from_f64(5e-324).is_none() && Exact::from_f64(1e300).is_none());
    for x in [0.2, -0.0035, 1234.5678, 3.0e-20] {
        assert_eq!(Exact::from_f64(x).unwrap().to_f64().to_bits(), x.to_bits());
    }
}

#[test]
fn checked_arithmetic_ordering_and_resolution() {
    assert_eq!(ex("0.1235").checked_sub(&ex("1/2000")).unwrap(), ex("0.123"));
    assert_eq!(ex("1/100").checked_mul(&ex("-130")).unwrap(), ex("-1.3"));
    assert_eq!(ex("1/3").checked_cmp(&ex("0.3333")), Some(Ordering::Greater));
    let big = Exact::from_ratio(i128::MAX, 1).unwrap();
    assert!(big.checked_add(&ex("1")).is_none() && big.checked_mul(&ex("2")).is_none());
    assert_eq!(ex("0.1235").on_grid(&ex("0.0005")), Some(true));
    assert_eq!(ex("0.1236").on_grid(&ex("0.0005")), Some(false));
}
