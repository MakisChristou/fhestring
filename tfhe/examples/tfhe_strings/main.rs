use ciphertext::fheasciichar::FheAsciiChar;
use string_method::StringMethod;
use tfhe::shortint::prelude::PARAM_MESSAGE_2_CARRY_2_KS_PBS;

use crate::args::StringArgs;
use crate::ciphertext::fhestring::FheString;
use crate::ciphertext::public_parameters::PublicParameters;
use std::time::Instant;

// All algorithms work with unpadded or padded strings
// Choose your string padding accordingly
const STRING_PADDING: usize = 1;

// This constant represents the upper bound of n given to the repeat algorithm
// Use a value that is higher than the intended repetitions but note that
// it increases time complexity of the algorithm in O(n^2)
const MAX_REPETITIONS: usize = 16;

// Max supported value is the maximum u8 value.
const MAX_FIND_LENGTH: usize = 255;

// Tfhe constants to have an 8bit value in our radix ciphertext
const MAX_BLOCKS: usize = 4;

mod args;
mod ciphertext;
mod client_key;
mod server_key;
mod string_method;
mod utils;

use client_key::MyClientKey;

fn main() {
    let string_args = StringArgs::from_args();

    assert!(
        string_args.n <= MAX_REPETITIONS,
        "Repeat method will not function correctly, increase MAX_REPETITIONS (max = 255)"
    );

    // Construct custom key types from tfhe-rs keys, based on the default parameters
    let my_client_key = MyClientKey::from_params(PARAM_MESSAGE_2_CARRY_2_KS_PBS, MAX_BLOCKS);
    let my_server_key = my_client_key.get_server_key();
    let public_parameters = my_client_key.get_public_parameters();

    let methods_to_test = [
        StringMethod::Contains,
        StringMethod::ContainsClear,
        StringMethod::EndsWith,
        StringMethod::EndsWithClear,
        StringMethod::EqIgnoreCase,
        StringMethod::Find,
        StringMethod::FindClear,
        StringMethod::IsEmpty,
        StringMethod::Len,
        StringMethod::Repeat,
        StringMethod::RepeatClear,
        StringMethod::Replace,
        StringMethod::ReplaceClear,
        StringMethod::ReplaceN,
        StringMethod::ReplaceNClear,
        StringMethod::Rfind,
        StringMethod::RfindClear,
        StringMethod::Rsplit,
        StringMethod::RsplitClear,
        StringMethod::RsplitOnce,
        StringMethod::RsplitOnceClear,
        StringMethod::RsplitN,
        StringMethod::RsplitNClear,
        StringMethod::RsplitTerminator,
        StringMethod::RsplitTerminatorClear,
        StringMethod::Split,
        StringMethod::SplitClear,
        StringMethod::SplitAsciiWhitespace,
        StringMethod::SplitInclusive,
        StringMethod::SplitInclusiveClear,
        StringMethod::SplitTerminator,
        StringMethod::SplitTerminatorClear,
        StringMethod::SplitN,
        StringMethod::SplitNClear,
        StringMethod::StartsWith,
        StringMethod::StartsWithClear,
        StringMethod::StripPrefix,
        StringMethod::StripPrefixClear,
        StringMethod::StripSuffix,
        StringMethod::StripSuffixClear,
        StringMethod::ToLower,
        StringMethod::ToUpper,
        StringMethod::Trim,
        StringMethod::TrimEnd,
        StringMethod::TrimStart,
        StringMethod::Concatenate,
        StringMethod::Lt,
        StringMethod::Le,
        StringMethod::Gt,
        StringMethod::Ge,
        StringMethod::Eq,
        StringMethod::Ne,
    ];

    for method in methods_to_test {
        let start = Instant::now();

        utils::run_fhe_str_method(
            &my_server_key,
            &my_client_key,
            &public_parameters,
            &string_args,
            &method,
        );

        let duration = start.elapsed();
        println!("{:?} {:?}", method, duration);
    }
}

#[cfg(test)]
mod test {
    use crate::ciphertext::fhesplit::FheSplit;
    use crate::ciphertext::fhestrip::FheStrip;
    use crate::server_key::MyServerKey;
    use crate::utils::{trim_str_vector, trim_vector};
    use crate::{FheAsciiChar, MyClientKey, PublicParameters, MAX_FIND_LENGTH, STRING_PADDING};
    use tfhe::shortint::prelude::PARAM_MESSAGE_2_CARRY_2_KS_PBS;

    fn setup_test() -> (MyClientKey, MyServerKey, PublicParameters) {
        let num_blocks = 4;

        // Construct custom key types from tfhe-rs keys, based on the default parameters
        let my_client_key = MyClientKey::from_params(PARAM_MESSAGE_2_CARRY_2_KS_PBS, num_blocks);
        let my_server_key = my_client_key.get_server_key();
        let public_parameters = my_client_key.get_public_parameters();

        (my_client_key, my_server_key, public_parameters)
    }

    #[test]
    fn valid_contains() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "awesomezamaisawesome";
        let needle_plain = "zama";

        let heistack =
            my_client_key.encrypt(heistack_plain, 3, &public_parameters, &my_server_key.key);
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.contains(&heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = heistack_plain.contains(needle_plain);

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn invalid_contains() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello world";
        let needle_plain = "zama";

        let heistack =
            my_client_key.encrypt(heistack_plain, 3, &public_parameters, &my_server_key.key);
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.contains(&heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = heistack_plain.contains(needle_plain);

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn invalid_ends_with() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello world";
        let needle_plain = "zama";

        let heistack = my_client_key.encrypt(
            heistack_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.ends_with(&heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = heistack_plain.ends_with(needle_plain);

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn valid_starts_with() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello world";
        let needle_plain = "hello";

        let heistack = my_client_key.encrypt(
            heistack_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.starts_with(&heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = heistack_plain.starts_with(needle_plain);

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn invalid_starts_with() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello world";
        let needle_plain = "zama";

        let heistack = my_client_key.encrypt(
            heistack_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.starts_with(&heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = heistack_plain.starts_with(needle_plain);

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn valid_ends_with() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello world";
        let needle_plain = "world";

        let heistack = my_client_key.encrypt(
            heistack_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.ends_with(&heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = heistack_plain.ends_with(needle_plain);

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn uppercase() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "zama IS awesome";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let my_string_upper = my_server_key.to_upper(&my_string, &public_parameters);

        let actual = my_client_key.decrypt(my_string_upper);
        let expected = my_string_plain.to_uppercase();

        assert_eq!(actual, expected);
    }

    #[test]
    fn repeat() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "abc";
        let n_plain = 3u8;

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let n = my_client_key.encrypt_char(n_plain);

        let my_string_upper = my_server_key.repeat(&my_string, n, &public_parameters);
        let actual = my_client_key.decrypt(my_string_upper);
        let expected = my_string_plain.repeat(n_plain.into());

        assert_eq!(actual, expected);
    }

    #[test]
    fn replace1() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "hello world world test";
        let from_plain = "world";
        let to_plain = "abc";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let from = my_client_key.encrypt_no_padding(from_plain);
        let to = my_client_key.encrypt_no_padding(to_plain);

        let my_new_string = my_server_key.replace(&my_string, &from, &to, &public_parameters);

        let actual = my_client_key.decrypt(my_new_string);
        let expected = my_string_plain.replace(from_plain, to_plain);

        assert_eq!(actual, expected);
    }

    #[test]
    fn replace2() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "hello abc abc test";
        let from_plain = "abc";
        let to_plain = "world";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let from = my_client_key.encrypt_no_padding(from_plain);
        let to = my_client_key.encrypt_no_padding(to_plain);

        let my_new_string = my_server_key.replace(&my_string, &from, &to, &public_parameters);

        let actual = my_client_key.decrypt(my_new_string);
        let expected = my_string_plain.replace(from_plain, to_plain);

        assert_eq!(actual, expected);
    }

    #[test]
    fn replacen() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "hello abc abc test";
        let from_plain = "abc";
        let to_plain = "world";
        let n_plain = 1u8;

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let from = my_client_key.encrypt_no_padding(from_plain);
        let to = my_client_key.encrypt_no_padding(to_plain);
        let n = my_client_key.encrypt_char(n_plain);

        let my_new_string = my_server_key.replacen(&my_string, &from, &to, n, &public_parameters);

        let actual = my_client_key.decrypt(my_new_string);
        let expected = my_string_plain.replacen(from_plain, to_plain, n_plain.into());

        assert_eq!(actual, expected);
    }

    #[test]
    fn lowercase() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "zama IS awesome";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let my_string_upper = my_server_key.to_lower(&my_string, &public_parameters);

        let actual = my_client_key.decrypt(my_string_upper);
        let expected = my_string_plain.to_lowercase();

        assert_eq!(actual, expected);
    }

    #[test]
    fn trim_end() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "ZA MA\n\t \r\x0C";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let my_string_upper = my_server_key.trim_end(&my_string, &public_parameters);

        let actual = my_client_key.decrypt(my_string_upper);
        let expected = my_string_plain.trim_end();

        assert_eq!(actual, expected);
    }

    #[test]
    fn do_not_trim_end() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "\nZA MA";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let my_string_upper = my_server_key.trim_end(&my_string, &public_parameters);

        let actual = my_client_key.decrypt(my_string_upper);
        let expected = my_string_plain.trim_end();

        assert_eq!(actual, expected);
    }

    #[test]
    fn trim_start() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "\nZA MA";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let my_string_upper = my_server_key.trim_start(&my_string, &public_parameters);

        let actual = my_client_key.decrypt(my_string_upper);
        let expected = my_string_plain.trim_start();

        assert_eq!(actual, expected);
    }

    #[test]
    fn trim() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "\nZA MA\n";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let my_string_upper = my_server_key.trim(&my_string, &public_parameters);

        let actual = my_client_key.decrypt(my_string_upper);
        let expected = my_string_plain.trim();

        assert_eq!(actual, expected);
    }

    #[test]
    fn is_empty() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "";
        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );

        let res = my_server_key.is_empty(&my_string, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);
        let expected = my_string_plain.is_empty();

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn is_not_empty() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "hello";
        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );

        let res = my_server_key.is_empty(&my_string, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);
        let expected = my_string_plain.is_empty();

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn len() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "hello world";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );

        let res = my_server_key.len(&my_string, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = my_string_plain.len();

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn rfind() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello abc abc test";
        let needle_plain = "abc";

        let heistack = my_client_key.encrypt(
            heistack_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.rfind(heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = heistack_plain.rfind(needle_plain).unwrap();

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn invalid_rfind() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello test";
        let needle_plain = "abc";

        let heistack = my_client_key.encrypt(
            heistack_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.rfind(heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        // The original algoritm returns None but since we don't have this luxury we will use a
        // placeholder value
        let _ = heistack_plain.rfind(needle_plain);

        assert_eq!(dec, MAX_FIND_LENGTH as u8);
    }

    #[test]
    #[should_panic(expected = "Maximum supported size for find reached")]
    fn unsupported_size_rfind() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello test".repeat(100);
        let needle_plain = "abc";

        let heistack = my_client_key.encrypt(
            &heistack_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let _ = my_server_key.rfind(heistack, &needle, &public_parameters);
    }

    #[test]
    fn find() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack_plain = "hello test";
        let needle_plain = "test";

        let heistack = my_client_key.encrypt(
            heistack_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let needle = my_client_key.encrypt_no_padding(needle_plain);

        let res = my_server_key.find(&heistack, &needle, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);

        let expected = heistack_plain.find(needle_plain).unwrap();

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn eq() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack1_plain = "hello test";
        let heistack2_plain = "hello test";

        let heistack1 = my_client_key.encrypt(
            heistack1_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let heistack2 = my_client_key.encrypt(
            heistack2_plain,
            STRING_PADDING + 20,
            &public_parameters,
            &my_server_key.key,
        );

        let res = my_server_key.eq(&heistack1, &heistack2, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);
        let expected = heistack1_plain.eq(heistack2_plain);

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn eq_ignore_case() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let heistack1_plain = "hello TEST";
        let heistack2_plain = "hello test";

        let heistack1 = my_client_key.encrypt(
            heistack1_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let heistack2 = my_client_key.encrypt(
            heistack2_plain,
            STRING_PADDING + 20,
            &public_parameters,
            &my_server_key.key,
        );

        let res = my_server_key.eq_ignore_case(&heistack1, &heistack2, &public_parameters);
        let dec: u8 = my_client_key.decrypt_char(&res);
        let expected = heistack1_plain.eq_ignore_ascii_case(heistack2_plain);

        assert_eq!(dec, expected as u8);
    }

    #[test]
    fn strip_prefix() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "HELLO test test HELLO";
        let pattern_plain = "HELLO";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);
        let fhe_strip = my_server_key.strip_prefix(&my_string, &pattern, &public_parameters);

        let (actual, _) = FheStrip::decrypt(fhe_strip, &my_client_key);

        let expected = my_string_plain.strip_prefix(pattern_plain).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn strip_suffix() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "HELLO test test HELLO";
        let pattern_plain = "HELLO";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);

        let fhe_strip = my_server_key.strip_suffix(my_string, &pattern, &public_parameters);

        let (actual, _) = FheStrip::decrypt(fhe_strip, &my_client_key);

        let expected = my_string_plain.strip_suffix(pattern_plain).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn dont_strip_suffix() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "HELLO test test HELLO";
        let pattern_plain = "WORLD";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);

        let fhe_strip = my_server_key.strip_suffix(my_string, &pattern, &public_parameters);

        let (_, pattern_found) = FheStrip::decrypt(fhe_strip, &my_client_key);

        // This is None but in our case the string is not modified
        let expected = my_string_plain.strip_suffix(pattern_plain);

        let expected_pattern_found = if let Some(_) = expected { true } else { false };

        assert_eq!(pattern_found, expected_pattern_found as u8);
    }

    #[test]
    fn dont_strip_prefix() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "HELLO test test HELLO";
        let pattern_plain = "WORLD";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern =
            my_client_key.encrypt(pattern_plain, 0, &public_parameters, &my_server_key.key);
        let fhe_strip =
            my_server_key.strip_prefix(&my_string, &pattern.get_bytes(), &public_parameters);

        let (_, pattern_found) = FheStrip::decrypt(fhe_strip, &my_client_key);

        // This is None but in our case the string is not modified
        let expected = my_string_plain.strip_prefix(pattern_plain);

        let expected_pattern_found = if let Some(_) = expected { true } else { false };

        assert_eq!(pattern_found, expected_pattern_found as u8);
    }

    #[test]
    fn concatenate() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string1_plain = "Hello, ";
        let my_string2_plain = "World!";

        let my_string1 = my_client_key.encrypt(
            my_string1_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let my_string2 = my_client_key.encrypt(
            my_string2_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let my_string_upper =
            my_server_key.concatenate(&my_string1, &my_string2, &public_parameters);

        let actual = my_client_key.decrypt(my_string_upper);
        assert_eq!(actual, format!("{}{}", my_string1_plain, my_string2_plain));
    }

    #[test]
    fn less_than() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain1 = "aaa";
        let my_string_plain2 = "aaaa";

        let heistack1 = my_client_key.encrypt(
            my_string_plain1,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let heistack2 = my_client_key.encrypt(
            my_string_plain2,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let actual = my_server_key.lt(&heistack1, &heistack2, &public_parameters);

        let deccrypted_actual: u8 = my_client_key.decrypt_char(&actual);

        let expected = (my_string_plain1 < my_string_plain2) as u8;

        assert_eq!(expected, deccrypted_actual);
    }

    #[test]
    fn less_equal() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain1 = "aaa";
        let my_string_plain2 = "aaaa";

        let heistack1 = my_client_key.encrypt(
            my_string_plain1,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let heistack2 = my_client_key.encrypt(
            my_string_plain2,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let actual = my_server_key.le(&heistack1, &heistack2, &public_parameters);

        let deccrypted_actual: u8 = my_client_key.decrypt_char(&actual);

        let expected = (my_string_plain1 <= my_string_plain2) as u8;

        assert_eq!(expected, deccrypted_actual);
    }

    #[test]
    fn greater_than() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain1 = "aaa";
        let my_string_plain2 = "aaaa";

        let heistack1 = my_client_key.encrypt(
            my_string_plain1,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let heistack2 = my_client_key.encrypt(
            my_string_plain2,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let actual = my_server_key.gt(&heistack1, &heistack2, &public_parameters);

        let deccrypted_actual: u8 = my_client_key.decrypt_char(&actual);

        let expected = (my_string_plain1 > my_string_plain2) as u8;

        assert_eq!(expected, deccrypted_actual);
    }

    #[test]
    fn greater_equal() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain1 = "aaa";
        let my_string_plain2 = "aaaa";

        let heistack1 = my_client_key.encrypt(
            my_string_plain1,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let heistack2 = my_client_key.encrypt(
            my_string_plain2,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let actual = my_server_key.ge(&heistack1, &heistack2, &public_parameters);

        let deccrypted_actual: u8 = my_client_key.decrypt_char(&actual);

        let expected = (my_string_plain1 >= my_string_plain2) as u8;

        assert_eq!(expected, deccrypted_actual);
    }

    #[test]
    fn split() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = " Mary had a";
        let pattern_plain = " ";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);

        let fhe_split = my_server_key.split(&my_string, &pattern, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);
        let expected: Vec<&str> = my_string_plain.split(pattern_plain).collect();

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }

    #[test]
    fn split_inclusive() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "Mary had a";
        let pattern_plain = " ";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);

        let fhe_split = my_server_key.split_inclusive(&my_string, &pattern, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);
        let expected: Vec<&str> = my_string_plain.split_inclusive(pattern_plain).collect();

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }

    #[test]
    fn split_terminator() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = ".A.B.";
        let pattern_plain = ".";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);

        let fhe_split = my_server_key.split_terminator(&my_string, &pattern, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);
        let expected: Vec<&str> = my_string_plain.split_terminator(pattern_plain).collect();

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }

    #[test]
    fn split_ascii_whitespace() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = " A\nB\t";
        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );

        let fhe_split = my_server_key.split_ascii_whitespace(&my_string, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);
        let expected: Vec<&str> = my_string_plain.split_ascii_whitespace().collect();

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }

    #[test]
    fn splitn() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = ".A.B.C.";
        let pattern_plain = ".";
        let n_plain = 2u8;

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);
        let n = FheAsciiChar::encrypt_trivial(n_plain, &public_parameters, &my_server_key.key);

        let fhe_split = my_server_key.splitn(&my_string, &pattern, n, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);

        let expected: Vec<&str> = my_string_plain
            .splitn(n_plain.into(), pattern_plain)
            .collect();

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }

    #[test]
    fn rsplit() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = ".A.B.C.";
        let pattern_plain = ".";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);

        let fhe_split = my_server_key.rsplit(&my_string, &pattern, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);
        let expected: Vec<&str> = my_string_plain.rsplit(pattern_plain).collect();

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }

    #[test]
    fn rsplit_once() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = ".A.B.C.";
        let pattern_plain = ".";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);

        let fhe_split = my_server_key.rsplit_once(&my_string, &pattern, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);
        let expected_tuple = my_string_plain.rsplit_once(pattern_plain).unwrap();
        let expected = vec![expected_tuple.1, expected_tuple.0];

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }

    #[test]
    fn rsplitn() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = ".A.B.C.";
        let pattern_plain = ".";
        let n_plain = 3u8;

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);
        let n = FheAsciiChar::encrypt_trivial(n_plain, &public_parameters, &my_server_key.key);

        let fhe_split = my_server_key.rsplitn(&my_string, &pattern, n, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);

        let expected: Vec<&str> = my_string_plain
            .rsplitn(n_plain.into(), pattern_plain)
            .collect();

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }

    #[test]
    fn rsplit_terminator() {
        let (my_client_key, my_server_key, public_parameters) = setup_test();

        let my_string_plain = "....A.B.C.";
        let pattern_plain = ".";

        let my_string = my_client_key.encrypt(
            my_string_plain,
            STRING_PADDING,
            &public_parameters,
            &my_server_key.key,
        );
        let pattern = my_client_key.encrypt_no_padding(pattern_plain);

        let fhe_split = my_server_key.rsplit_terminator(&my_string, &pattern, &public_parameters);
        let plain_split = FheSplit::decrypt(fhe_split, &my_client_key);

        let expected: Vec<&str> = my_string_plain.rsplit_terminator(pattern_plain).collect();

        let plain_split = trim_vector(plain_split.0);
        let expected = trim_str_vector(expected);
        assert_eq!(plain_split, expected);
    }
}
