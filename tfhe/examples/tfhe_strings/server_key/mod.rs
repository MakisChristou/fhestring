use crate::ciphertext::fheasciichar::FheAsciiChar;
use crate::ciphertext::fhestring::{Comparison, FheString};
use crate::ciphertext::fhestrip::FheStrip;
use crate::ciphertext::public_parameters::PublicParameters;
use crate::client_key::MyClientKey;
use crate::utils::{self, abs_difference};
use crate::{MAX_FIND_LENGTH, MAX_REPETITIONS};
use serde::{Deserialize, Serialize};

pub mod split;
pub mod trim;

#[derive(Serialize, Deserialize, Clone)]
pub struct MyServerKey {
    pub key: tfhe::integer::ServerKey,
}

impl MyServerKey {
    /// Creates a new `MyServerKey` instance from a given `ServerKey`.
    ///
    /// # Arguments
    /// * `server_key`: tfhe::integer::ServerKey - The server key to be used in FHE operations.
    ///
    /// # Returns
    /// `MyServerKey` - A new `MyServerKey` instance.
    pub fn new(server_key: tfhe::integer::ServerKey) -> Self {
        MyServerKey { key: server_key }
    }

    /// Creates a new `MyServerKey` instance from a given `MyClientKey`.
    ///
    /// # Arguments
    /// * `my_client_key`: MyClientKey - The client key used to extract the server key.
    ///
    /// # Returns
    /// `MyServerKey` - A new `MyServerKey` instance constructed from the server key derived from
    ///  `my_client_key`.
    pub fn _from_client_key(my_client_key: MyClientKey) -> Self {
        my_client_key.get_server_key()
    }

    /// Converts all lowercase characters in a given `FheString` to uppercase.
    ///
    /// # Arguments
    /// * `string`: &FheString - The FheString to be converted.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheString` - An uppercase version of the input string.
    /// Example:
    ///
    /// ```
    /// let my_string_plain = "zama IS awesome";
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let my_string_upper = my_server_key.to_upper(&my_string, &public_parameters);
    /// let actual = my_client_key.decrypt(my_string_upper);
    ///
    /// assert_eq!(actual, "ZAMA IS AWESOME");
    /// ```
    pub fn to_upper(&self, string: &FheString, public_parameters: &PublicParameters) -> FheString {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);

        let bytes = string
            .iter()
            .map(|b| {
                let is_not_lowercase = b
                    .is_lowercase(&self.key, public_parameters)
                    .flip(&self.key, public_parameters);
                b.sub(
                    &self.key,
                    &is_not_lowercase.if_then_else(&self.key, &zero, &string.get_cst()),
                )
            })
            .collect::<Vec<FheAsciiChar>>();

        let cst = string.get_cst();

        FheString::new(bytes, cst)
    }

    /// Converts all uppercase characters in a given `FheString` to lowercase.
    ///
    /// # Arguments
    /// * `string`: &FheString - The FheString to be converted.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheString` - A lowercase version of the input string.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "zama IS awesome";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let my_string_upper = my_server_key.to_lower(&my_string, &public_parameters);
    /// let actual = my_client_key.decrypt(my_string_upper);
    ///
    /// assert_eq!(actual, "zama is awesome");
    /// ```
    pub fn to_lower(&self, string: &FheString, public_parameters: &PublicParameters) -> FheString {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);

        let bytes = string
            .iter()
            .map(|b| {
                let is_not_uppercase = b
                    .is_uppercase(&self.key, public_parameters)
                    .flip(&self.key, public_parameters);
                b.add(
                    &self.key,
                    &is_not_uppercase.if_then_else(&self.key, &zero, &string.get_cst()),
                )
            })
            .collect::<Vec<FheAsciiChar>>();
        let cst = string.get_cst();

        FheString::new(bytes, cst)
    }

    /// Checks if a given `FheString` contains a specified pattern.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to search within.
    /// * `needle`: &Vec<FheAsciiChar> - The unpadded pattern to search for.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - Encrypted 1 if the pattern is found, otherwise encrypted 0.
    ///
    /// # Example
    /// ```
    /// let heistack_plain = "awesome zama is awesome";
    /// let needle_plain = "zama";
    /// let heistack = my_client_key.encrypt(heistack_plain, 3, &public_parameters, &my_server_key.key);
    /// let needle = my_client_key.encrypt_no_padding(needle_plain);
    ///
    /// let res = my_server_key.contains(&heistack, &needle, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn contains(
        &self,
        string: &FheString,
        needle: &Vec<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        if string.is_empty() && needle.is_empty() {
            return FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        }
        let mut result = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let end = string.len().checked_sub(needle.len());

        match end {
            Some(end_of_pattern) => {
                // If pattern and string have the same size and are equal
                // this is needed to actually iterate the loop
                // let end_of_pattern = utils::adjust_end_of_pattern(end_of_pattern);

                for i in 0..=end_of_pattern {
                    let mut current_result = one.clone();
                    for (j, needle_char) in needle.iter().enumerate() {
                        let eql = string[i + j].eq(&self.key, needle_char);
                        current_result = current_result.bitand(&self.key, &eql);
                    }
                    result = result.bitor(&self.key, &current_result);
                }
                result
            }
            None => FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key),
        }
    }

    /// Checks if a given `FheString` contains a specified plaintext pattern.
    ///
    /// Same as `contains` but with plaintext pattern.
    /// # Example
    /// ```
    /// let (my_client_key, my_server_key, public_parameters) = setup_test();
    ///
    /// let heistack_plain = "awesome zama is awesome";
    /// let needle_plain = "zama";
    /// let heistack = my_client_key.encrypt(heistack_plain, 3, &public_parameters, &my_server_key.key);
    /// let res = my_server_key.contains_clear(&heistack, &needle_plain, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn contains_clear(
        &self,
        string: &FheString,
        clear_needle: &str,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let needle = clear_needle
            .as_bytes()
            .iter()
            .map(|b| FheAsciiChar::encrypt_trivial(*b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();

        self.contains(string, &needle, public_parameters)
    }

    /// Checks if a given `FheString` ends with a specified pattern, considering padding.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to check.
    /// * `pattern`: &Vec<FheAsciiChar> - The unpadded pattern to compare against.
    /// * `padding`: usize - The padding size to consider at the end of the string.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - Encrypted 1 if the string ends with the pattern, otherwise encrypted 0.
    /// # Example
    /// ```
    /// let heistack_plain = "hello world";
    /// let needle_plain = "world";
    ///
    /// let heistack = my_client_key.encrypt(
    ///     heistack_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let needle = my_client_key.encrypt_no_padding(needle_plain);
    ///
    /// let res = my_server_key.ends_with(&heistack, &needle, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn ends_with(
        &self,
        string: &FheString,
        needle: &Vec<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        if string.is_empty() && needle.is_empty() {
            return FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        }
        let mut result = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let end = string.len().checked_sub(needle.len());

        match end {
            Some(end_of_pattern) => {
                for i in 0..=end_of_pattern {
                    let mut current_result = one.clone();
                    let mut are_all_comparison_chars_non_zero = one.clone();

                    for (j, needle_char) in needle.iter().enumerate() {
                        let eql = string[i + j].eq(&self.key, needle_char);
                        current_result = current_result.bitand(&self.key, &eql);

                        // If we encounter padding we should ignore the result
                        let is_char_not_zero = string[i + j].ne(&self.key, &zero);
                        are_all_comparison_chars_non_zero =
                            are_all_comparison_chars_non_zero.bitand(&self.key, &is_char_not_zero);
                    }
                    // Use the last result that has not encrountered padding
                    result = are_all_comparison_chars_non_zero.if_then_else(
                        &self.key,
                        &current_result,
                        &result,
                    );
                }
                result
            }
            None => FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key),
        }
    }

    /// Checks if a given `FheString` ends with a specified plaintext pattern, considering padding.
    ///
    /// Same as `ends_with` but with plaintext pattern  .
    /// Example:
    /// ```
    /// let heistack_plain = "hello world";
    /// let needle_plain = "world";
    ///
    /// let heistack = my_client_key.encrypt(
    ///     heistack_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.ends_with_clear(&heistack, &needle_plain, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn ends_with_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let pattern = clear_pattern
            .as_bytes()
            .iter()
            .map(|b| FheAsciiChar::encrypt_trivial(*b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self.ends_with(string, &pattern, public_parameters)
    }

    /// Checks if a given `FheString` starts with a specified pattern.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to check.
    /// * `pattern`: &[FheAsciiChar] - The unpadded pattern to compare against.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - Encrypted 1 if the string starts with the pattern, otherwise encrypted 0.
    ///
    /// # Example
    /// ```
    /// let heistack_plain = "hello world";
    /// let needle_plain = "hello";
    ///
    /// let heistack = my_client_key.encrypt(
    ///     heistack_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let needle = my_client_key.encrypt_no_padding(needle_plain);
    /// let res = my_server_key.starts_with(&heistack, &needle, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```    
    pub fn starts_with(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let mut result = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let end_of_pattern = std::cmp::min(pattern.len(), string.len());

        if pattern.len() > string.len() {
            return FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        }

        if string.is_empty() && pattern.is_empty() {
            return FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        } else if string.is_empty() && !pattern.is_empty() {
            return FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        }

        for (string_char, pattern_char) in string.iter().take(end_of_pattern).zip(pattern) {
            let eql = string_char.eq(&self.key, pattern_char);
            result = result.bitand(&self.key, &eql);
        }

        result
    }

    /// Checks if a given `FheString` starts with a specified plaintext pattern.
    ///
    /// Same as `starts_with` but with plaintext pattern.
    ///
    /// # Example
    /// ```
    /// let heistack_plain = "hello world";
    /// let needle_plain = "hello";
    ///
    /// let heistack = my_client_key.encrypt(
    ///     heistack_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.starts_with_clear(&heistack, &needle_plain, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```    
    pub fn starts_with_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let pattern = clear_pattern
            .as_bytes()
            .iter()
            .map(|b| FheAsciiChar::encrypt_trivial(*b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self.starts_with(string, &pattern, public_parameters)
    }

    /// Checks if a given `FheString` is empty.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to check.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - Encrypted 1 if the string is empty, otherwise encrypted 0.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.is_empty(&my_string, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn is_empty(
        &self,
        string: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);

        if string.is_empty() {
            return one;
        }

        let mut result = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);

        for i in 0..string.len() {
            let eql = string[i].eq(&self.key, &zero);
            result = result.bitand(&self.key, &eql);
        }

        result
    }

    /// Computes the length of a given `FheString`.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string whose length is to be computed.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - The encrypted length of the string, without the padding
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "hello world";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.len(&my_string, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 11u8);
    /// ```
    pub fn len(&self, string: &FheString, public_parameters: &PublicParameters) -> FheAsciiChar {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);

        if string.is_empty() {
            return zero;
        }

        let mut result = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);

        for i in 0..string.len() {
            let is_not_zero = string[i].ne(&self.key, &zero);
            result = result.add(&self.key, &is_not_zero);
        }

        result
    }

    /// Repeats a given `FheString` a specified number of times for a max number
    /// of MAX_REPETITIONS. Max valid repetitions value is 255u8.
    ///
    /// Same as `repeat` but with plaintext repetitions.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "abc";
    /// let n_plain = 3u8;
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let my_string_upper =
    ///     my_server_key.repeat_clear(&my_string, n_plain.into(), &public_parameters);
    /// let actual = my_client_key.decrypt(my_string_upper);
    ///
    /// assert_eq!(actual, "abcabcabc");
    /// ```
    pub fn repeat_clear(
        &self,
        string: &FheString,
        repetitions: usize,
        public_parameters: &PublicParameters,
    ) -> FheString {
        let mut result = string.clone();
        let end = repetitions.checked_sub(1);

        match end {
            Some(end_of_pattern) => {
                for _ in 0..end_of_pattern {
                    result.append(string.clone());
                }

                utils::bubble_zeroes_right(result, &self.key, public_parameters)
            }

            None => FheString::from_vec(vec![], public_parameters, &self.key),
        }
    }

    /// Repeats a given `FheString` a specified number of times for a max number
    /// of MAX_REPETITIONS. Max valid repetitions value is 255u8.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to be repeated.
    /// * `repetitions`: FheAsciiChar - Encrypted number of times to repeat the string.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheString` - The repeated string.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "abc";
    /// let n_plain = 3u8;
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let n = my_client_key.encrypt_char(n_plain);
    /// let my_string_upper = my_server_key.repeat(&my_string, n, &public_parameters);
    /// let actual = my_client_key.decrypt(my_string_upper);
    ///
    /// assert_eq!(actual, "abcabcabc");
    /// ```
    pub fn repeat(
        &self,
        string: &FheString,
        repetitions: FheAsciiChar,
        public_parameters: &PublicParameters,
    ) -> FheString {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let mut result = FheString::from_vec(
            vec![zero.clone(); MAX_REPETITIONS * string.len()],
            public_parameters,
            &self.key,
        );
        let str_len = string.len();

        for i in 0..MAX_REPETITIONS {
            let enc_i = FheAsciiChar::encrypt_trivial(i as u8, public_parameters, &self.key);
            let copy_flag = enc_i.lt(&self.key, &repetitions);

            for j in 0..str_len {
                result[i * str_len + j] = copy_flag.if_then_else(&self.key, &string[j], &zero);
            }
        }

        utils::bubble_zeroes_right(result, &self.key, public_parameters)
    }

    /// Replaces occurrences of a pattern in a given `FheString` with another pattern.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string in which replacements are to be made.
    /// * `from`: &Vec<FheAsciiChar> - The unpadded pattern to be replaced.
    /// * `to`: &Vec<FheAsciiChar> - The unpadded pattern to replace with.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheString` - The string with replacements made.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "hello world world test";
    /// let from_plain = "world";
    /// let to_plain = "abc";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let from = my_client_key.encrypt_no_padding(from_plain);
    /// let to = my_client_key.encrypt_no_padding(to_plain);
    ///
    /// let my_new_string = my_server_key.replace(&my_string, &from, &to, &public_parameters);
    /// let actual = my_client_key.decrypt(my_new_string);
    ///
    /// assert_eq!(actual, "hello abc abc test");
    /// ```
    pub fn replace(
        &self,
        string: &FheString,
        from: &Vec<FheAsciiChar>,
        to: &Vec<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheString {
        let n = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        if from.len() >= to.len() {
            Self::handle_longer_from(
                string.clone(),
                from.clone(),
                to.clone(),
                n,
                false,
                &self.key,
                public_parameters,
            )
        } else {
            Self::handle_shorter_from(
                string.clone(),
                from.clone(),
                to.clone(),
                n,
                false,
                &self.key,
                public_parameters,
            )
        }
    }

    /// Replaces occurrences of a plaintext pattern in a given `FheString` with another plaintext
    /// pattern.
    ///
    /// Same as `replace` but with plaintext patterns.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "hello world world test";
    /// let from_plain = "world";
    /// let to_plain = "abc";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let my_new_string =
    ///     my_server_key.replace_clear(&my_string, &from_plain, &to_plain, &public_parameters);
    /// let actual = my_client_key.decrypt(my_new_string);
    ///
    /// assert_eq!(actual, "hello abc abc test");
    /// ```
    pub fn replace_clear(
        &self,
        string: &FheString,
        clear_from: &str,
        clear_to: &str,
        public_parameters: &PublicParameters,
    ) -> FheString {
        let from = clear_from
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();

        let to = clear_to
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();

        self.replace(string, &from, &to, public_parameters)
    }

    /// Finds the last occurrence of a pattern in a given `FheString`.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to search.
    /// * `pattern`: &Vec<FheAsciiChar> - The unpadded pattern to find.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - The encrypted position of the last occurrence of the pattern,
    /// or encrypted MAX_FIND_LENGTH if not found
    ///
    /// # Example:
    /// ```
    /// let heistack_plain = "hello abc abc test";
    /// let needle_plain = "abc";
    ///
    /// let heistack = my_client_key.encrypt(
    ///     heistack_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let needle = my_client_key.encrypt_no_padding(needle_plain);
    /// let res = my_server_key.rfind(heistack, &needle, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 10u8);
    /// ```
    pub fn rfind(
        &self,
        mut string: FheString,
        pattern: &Vec<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);

        // Quick solution to fix a no padding issue
        string.push(zero.clone());

        let mut pattern_position =
            FheAsciiChar::encrypt_trivial(MAX_FIND_LENGTH as u8, public_parameters, &self.key);

        if string.len() >= MAX_FIND_LENGTH + pattern.len() {
            panic!("Maximum supported size for find reached");
        }

        // Handle edge case
        if pattern.is_empty() {
            let mut last_non_zero_position = zero.clone();

            // Find the last char position that is non \0
            for i in 0..string.len() {
                let is_not_zero = string[i].ne(&self.key, &zero);
                let enc_i =
                    FheAsciiChar::encrypt_trivial((i + 1) as u8, public_parameters, &self.key);
                last_non_zero_position =
                    is_not_zero.if_then_else(&self.key, &enc_i, &last_non_zero_position);
            }

            return last_non_zero_position;
        }

        let end = string.len().checked_sub(pattern.len());

        match end {
            Some(end_of_pattern) => {
                // If pattern and string have the same size and are equal
                // this is needed to actually iterate the loop
                let end_of_pattern = utils::adjust_end_of_pattern(end_of_pattern);

                // Search for pattern
                for i in 0..end_of_pattern {
                    let mut pattern_found_flag = one.clone();

                    // This is okay since pattern.len() <= string.bytes.len()
                    for (j, pattern_char) in pattern.iter().enumerate() {
                        pattern_found_flag = pattern_found_flag
                            .bitand(&self.key, &pattern_char.eq(&self.key, &string[i + j]));
                    }

                    let enc_i =
                        FheAsciiChar::encrypt_trivial(i as u8, public_parameters, &self.key);
                    pattern_position =
                        pattern_found_flag.if_then_else(&self.key, &enc_i, &pattern_position);
                }

                pattern_position
            }
            None => FheAsciiChar::encrypt_trivial(255u8, public_parameters, &self.key),
        }
    }

    /// Finds the last occurrence of a plaintext pattern in a given `FheString`.
    ///
    /// Same as `rfind` but with a plaintext pattern.
    ///
    /// # Example:
    /// ```
    /// let heistack_plain = "hello abc abc test";
    /// let needle_plain = "abc";
    ///
    /// let heistack = my_client_key.encrypt(
    ///     heistack_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.rfind_clear(&heistack, &needle_plain, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 10u8);
    /// ```
    pub fn rfind_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();

        self.rfind(string.clone(), &pattern, public_parameters)
    }

    // The "easy" case
    fn handle_longer_from(
        mut bytes: FheString,
        from: Vec<FheAsciiChar>,
        mut to: Vec<FheAsciiChar>,
        n: FheAsciiChar,
        use_counter: bool,
        server_key: &tfhe::integer::ServerKey,
        public_parameters: &PublicParameters,
    ) -> FheString {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, server_key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, server_key);

        // Quick solution to fix a no padding issue
        bytes.push(zero.clone());

        let size_difference = abs_difference(from.len(), to.len());
        let mut counter = FheAsciiChar::encrypt_trivial(0u8, public_parameters, server_key);

        // Pad to with zeroes
        for _ in 0..size_difference {
            to.push(zero.clone());
        }

        let mut result = bytes.clone();

        if from.len() <= result.len() {
            // If pattern and string have the same size and are equal
            // this is needed to actually iterate the loop
            let end_of_pattern = utils::adjust_end_of_pattern(result.len() - from.len());

            // Replace from wih to
            for i in 0..end_of_pattern {
                let mut pattern_found_flag = one.clone();

                for j in 0..from.len() {
                    pattern_found_flag = pattern_found_flag
                        .bitand(server_key, &from[j].eq(server_key, &bytes[i + j]));
                }

                // Stop replacing after n encounters of from
                if use_counter {
                    counter = counter.add(server_key, &pattern_found_flag);
                    let keep_replacing = n.ge(server_key, &counter);
                    pattern_found_flag = pattern_found_flag.bitand(server_key, &keep_replacing);
                }

                for k in 0..to.len() {
                    result[i + k] =
                        pattern_found_flag.if_then_else(server_key, &to[k], &result[i + k]);
                }
            }
        }

        utils::bubble_zeroes_right(result, server_key, public_parameters)
    }

    // The "hard" case
    fn handle_shorter_from(
        mut bytes: FheString,
        from: Vec<FheAsciiChar>,
        to: Vec<FheAsciiChar>,
        n: FheAsciiChar,
        use_counter: bool,
        server_key: &tfhe::integer::ServerKey,
        public_parameters: &PublicParameters,
    ) -> FheString {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, server_key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, server_key);

        // Quick solution to fix a no padding issue
        bytes.push(zero.clone());

        let size_difference = abs_difference(from.len(), to.len());
        let mut counter = FheAsciiChar::encrypt_trivial(0u8, public_parameters, server_key);

        let max_possible_output_len = if bytes.is_empty() {
            to.len()
        } else {
            to.len() * bytes.len() + bytes.len()
        };

        // This implies that we match all characters
        let max_possible_output_len = if from.is_empty() {
            (bytes.len() + (bytes.len() + 1) * to.len()) + 1
        } else {
            max_possible_output_len
        };

        let mut result = bytes.clone();

        for _ in 0..max_possible_output_len - bytes.len() {
            result.push(zero.clone());
        }

        let mut copy_buffer = vec![zero.clone(); max_possible_output_len];
        // This is used to ignore invalid pattern found flags
        // This happens if for example we replace e with test, the e in test will match the pattern
        // but its invalid
        let mut ignore_pattern_mask = vec![one.clone(); max_possible_output_len];

        // Replace from wih to
        for i in 0..result.len() - to.len() {
            let mut pattern_found_flag = one.clone();

            for j in 0..from.len() {
                pattern_found_flag =
                    pattern_found_flag.bitand(server_key, &from[j].eq(server_key, &result[i + j]));
                pattern_found_flag =
                    pattern_found_flag.bitand(server_key, &ignore_pattern_mask[i + j]);
            }

            // Handle spacial case where from is empty which means that it matches all characters
            // I know its ugly but it works
            if from.is_empty() {
                if i % (to.len() + 1) == 0 {
                    pattern_found_flag = one.clone();
                } else {
                    pattern_found_flag = zero.clone();
                }
            }

            // Stop replacing after n encounters of from
            if use_counter {
                counter = counter.add(server_key, &pattern_found_flag);
                let keep_replacing = n.ge(server_key, &counter);
                pattern_found_flag = pattern_found_flag.bitand(server_key, &keep_replacing);
            }

            // Copy original string to buffer
            for k in 0..max_possible_output_len {
                copy_buffer[k] = pattern_found_flag.if_then_else(server_key, &result[k], &zero);
            }

            // Replace from with to
            for k in 0..to.len() {
                result[i + k] = pattern_found_flag.if_then_else(server_key, &to[k], &result[i + k]);
                ignore_pattern_mask[i + k] = ignore_pattern_mask[i + k].bitand(
                    server_key,
                    &pattern_found_flag.if_then_else(server_key, &zero, &one),
                );
            }

            // Fix the result buffer by copying back the rest of the string
            for k in i + to.len()..max_possible_output_len {
                result[k] = pattern_found_flag.if_then_else(
                    server_key,
                    &copy_buffer[k - size_difference],
                    &result[k],
                );
            }
        }
        result
    }

    /// Finds the first occurrence of a pattern in a given `FheString`.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to search.
    /// * `pattern`: &Vec<FheAsciiChar> - The unpadded pattern to find.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - The encrypted position of the first occurrence of the pattern,
    ///  or encrypted MAX_FIND_LENGTH if not found
    ///
    /// # Example:
    /// ```
    /// let heistack_plain = "hello test";
    /// let needle_plain = "test";
    ///
    /// let heistack = my_client_key.encrypt(
    ///     heistack_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let needle = my_client_key.encrypt_no_padding(needle_plain);
    /// let res = my_server_key.find(&heistack, &needle, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 6u8);
    /// ```
    pub fn find(
        &self,
        string: &FheString,
        pattern: &Vec<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        // Edge case: If both are empty return found at position 0
        if string.is_empty() && pattern.is_empty() {
            return FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        }

        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let mut pattern_position =
            FheAsciiChar::encrypt_trivial(MAX_FIND_LENGTH as u8, public_parameters, &self.key);

        if string.len() >= MAX_FIND_LENGTH + pattern.len() {
            panic!("Maximum supported size for find reached");
        }

        let end = string.len().checked_sub(pattern.len());

        match end {
            Some(end_of_pattern) => {
                // Search for pattern
                for i in (0..=end_of_pattern).rev() {
                    let mut pattern_found_flag = one.clone();

                    // This is okay since the pattern here is <= string.bytes.len()
                    for j in (0..pattern.len()).rev() {
                        pattern_found_flag = pattern_found_flag
                            .bitand(&self.key, &pattern[j].eq(&self.key, &string[i + j]));
                    }

                    let enc_i =
                        FheAsciiChar::encrypt_trivial(i as u8, public_parameters, &self.key);
                    pattern_position =
                        pattern_found_flag.if_then_else(&self.key, &enc_i, &pattern_position);
                }

                pattern_position
            }
            None => FheAsciiChar::encrypt_trivial(255u8, public_parameters, &self.key),
        }
    }

    /// Finds the first occurrence of a plaintext pattern in a given `FheString`.
    ///
    /// Same as `find` but with a plaintext pattern.
    ///
    /// # Example:
    /// ```
    /// let heistack_plain = "hello test";
    /// let needle_plain = "test";
    ///
    /// let heistack = my_client_key.encrypt(
    ///     heistack_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let res = my_server_key.find_clear(&heistack, &needle_plain, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 6u8);
    /// ```
    pub fn find_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();

        self.find(string, &pattern, public_parameters)
    }

    /// Checks if two `FheString` instances are equal.
    ///
    /// # Arguments
    /// * `string`: &FheString - The first string to compare.
    /// * `other`: &FheString - The second string to compare.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - Encrypted 1 if strings are equal, otherwise encrypted 0.
    ///
    /// # Example:
    /// ```
    /// let heistack1_plain = "hello test";
    /// let heistack2_plain = "hello test";
    ///
    /// let heistack1 = my_client_key.encrypt(
    ///     heistack1_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let heistack2 = my_client_key.encrypt(
    ///     heistack2_plain,
    ///     STRING_PADDING + 20,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.eq(&heistack1, &heistack2, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn eq(
        &self,
        string: &FheString,
        other: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let mut is_eq = one.clone();
        let min_length = usize::min(string.len(), other.len());

        let len1 = self.len(string, public_parameters);
        let len2 = self.len(other, public_parameters);
        let are_lengths_not_eql = len1.ne(&self.key, &len2);

        for i in 0..min_length {
            let are_equal = string[i].eq(&self.key, &other[i]);
            let is_first_eq_zero = string[i].eq(&self.key, &zero);
            let is_second_eq_zero = other[i].eq(&self.key, &zero);

            let res = is_first_eq_zero.bitand(&self.key, &is_second_eq_zero);
            let res = res.bitor(&self.key, &are_equal);

            is_eq = is_eq.bitand(&self.key, &res);
        }
        // If strings have actual lengths that are not equal then they can never be equal
        are_lengths_not_eql.if_then_else(&self.key, &zero, &is_eq)
    }

    /// Checks if two `FheString` instances are not equal.
    ///
    /// Same as `eq` but returns true if strings are not equal.
    ///
    /// # Example:
    /// ```
    /// let heistack1_plain = "hello test";
    /// let heistack2_plain = "hello test";
    ///
    /// let heistack1 = my_client_key.encrypt(
    ///     heistack1_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let heistack2 = my_client_key.encrypt(
    ///     heistack2_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.ne(&heistack1, &heistack2, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 0u8);
    /// ```
    pub fn ne(
        &self,
        string: &FheString,
        other: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let res = self.eq(string, other, public_parameters);
        res.flip(&self.key, public_parameters)
    }

    /// Checks if two `FheString` instances are equal, ignoring case.
    ///
    /// # Arguments
    /// * `string`: &FheString - The first string to compare.
    /// * `other`: &FheString - The second string to compare.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - Encrypted 1 if strings are equal ignoring case, otherwise encrypted 0.
    ///
    /// # Example:
    /// ```
    /// let heistack1_plain = "hello TEST";
    /// let heistack2_plain = "hello test";
    ///
    /// let heistack1 = my_client_key.encrypt(
    ///     heistack1_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let heistack2 = my_client_key.encrypt(
    ///     heistack2_plain,
    ///     STRING_PADDING + 20,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.eq_ignore_case(&heistack1, &heistack2, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn eq_ignore_case(
        &self,
        string: &FheString,
        other: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let self_lowercase = self.to_lower(string, public_parameters);
        let other_lowercase = self.to_lower(other, public_parameters);

        self.eq(&self_lowercase, &other_lowercase, public_parameters)
    }

    /// Strips a specified pattern from the beginning of a `FheString`.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to modify.
    /// * `pattern`: &Vec<FheAsciiChar> - The unpadded pattern to strip.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheStrip` - A struct containing the new `FheString` with the pattern stripped from the
    /// beginning if found, and a boolean flag indicating whether the pattern was found or not.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "HELLO test test HELLO";
    /// let pattern_plain = "HELLO";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let pattern = my_client_key.encrypt_no_padding(pattern_plain);
    /// let fhe_strip = my_server_key.strip_prefix(&my_string, &pattern, &public_parameters);
    /// let (actual, _) = FheStrip::decrypt(fhe_strip, &my_client_key);
    ///
    /// assert_eq!(actual, " test test HELLO");
    /// ```
    pub fn strip_prefix(
        &self,
        string: &FheString,
        pattern: &Vec<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheStrip {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let mut result = string.clone();
        let mut pattern_found_flag = one.clone();

        let end = std::cmp::min(pattern.len(), result.len());

        // If pattern is bigger than a padded string by definition it cannot be found
        if pattern.len() > result.len() {
            return FheStrip::new(result, zero);
        }

        // Special Case: Either string is empty, or pattern is empty, or both
        if end == 0 {
            // If pattern is "" then its considered always found even for an empty string
            if pattern.is_empty() {
                pattern_found_flag = one.clone();
            }
            // In this case the pattern is considered never found
            else if !pattern.is_empty() && string.is_empty() {
                pattern_found_flag = zero.clone();
            }
        }

        for j in 0..end {
            pattern_found_flag =
                pattern_found_flag.bitand(&self.key, &pattern[j].eq(&self.key, &result[j]));
        }

        for result_char in result.iter_mut().take(pattern.len()) {
            *result_char = pattern_found_flag.if_then_else(&self.key, &zero, result_char);
        }

        let string = utils::bubble_zeroes_right(result, &self.key, public_parameters);
        FheStrip::new(string, pattern_found_flag)
    }

    /// Strips a specified pattern from the end of a `FheString`.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string to modify.
    /// * `pattern`: &Vec<FheAsciiChar> - The padded pattern to strip.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheStrip` - A struct containing the new `FheString` with the pattern stripped from the
    /// ending if found, and a boolean flag indicating whether the pattern was found or not.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "HELLO test test HELLO";
    /// let pattern_plain = "HELLO";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let pattern = my_client_key.encrypt_no_padding(pattern_plain);
    ///
    /// let fhe_strip = my_server_key.strip_suffix(my_string, &pattern, &public_parameters);
    ///
    /// let (actual, flag) = FheStrip::decrypt(fhe_strip, &my_client_key);
    ///
    /// assert_eq!(actual, "HELLO test test ");
    /// assert_eq!(flag, 1u8);
    /// ```
    pub fn strip_suffix(
        &self,
        mut string: FheString,
        needle: &Vec<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheStrip {
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let end = string.len().checked_sub(needle.len());
        let two_five_five = FheAsciiChar::encrypt_trivial(255u8, public_parameters, &self.key);

        let mut pattern_position =
            FheAsciiChar::encrypt_trivial(255u8, public_parameters, &self.key);

        match end {
            Some(end_of_pattern) => {
                for i in 0..=end_of_pattern {
                    let mut pattern_found = one.clone();
                    let mut are_all_comparison_chars_non_zero = one.clone();
                    let enc_i =
                        FheAsciiChar::encrypt_trivial(i as u8, public_parameters, &self.key);

                    for (j, needle_char) in needle.iter().enumerate() {
                        let eql = string[i + j].eq(&self.key, needle_char);
                        pattern_found = pattern_found.bitand(&self.key, &eql);

                        // If we encounter padding we should ignore the result
                        let is_char_not_zero = string[i + j].ne(&self.key, &zero);
                        are_all_comparison_chars_non_zero =
                            are_all_comparison_chars_non_zero.bitand(&self.key, &is_char_not_zero);
                    }

                    let current_result =
                        pattern_found.if_then_else(&self.key, &enc_i, &two_five_five);

                    // Use the last result that has not encrountered padding
                    pattern_position = are_all_comparison_chars_non_zero.if_then_else(
                        &self.key,
                        &current_result,
                        &pattern_position,
                    );
                }

                let should_strip_suffix = pattern_position.ne(&self.key, &two_five_five);

                for i in 0..=end_of_pattern {
                    let enc_i =
                        FheAsciiChar::encrypt_trivial(i as u8, public_parameters, &self.key);

                    let should_mask_pattern = enc_i.eq(&self.key, &pattern_position);

                    for (j, _) in needle.iter().enumerate() {
                        string[i + j] =
                            should_mask_pattern.if_then_else(&self.key, &zero, &string[i + j]);
                    }
                }

                FheStrip::new(string, should_strip_suffix)
            }
            None => FheStrip::new(string, zero),
        }
    }

    /// Strips a plaintext pattern from the beginning of a `FheString`.
    ///
    /// Same as `strip_prefix` but with a plaintext pattern.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "HELLO test test HELLO";
    /// let pattern_plain = "HELLO";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let fhe_strip =
    ///     my_server_key.strip_prefix_clear(&my_string, &pattern_plain, &public_parameters);
    /// let (actual, flag) = FheStrip::decrypt(fhe_strip, &my_client_key);
    ///
    /// assert_eq!(actual, " test test HELLO");
    /// assert_eq!(flag, 1u8);
    /// ```
    pub fn strip_prefix_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheStrip {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self.strip_prefix(string, &pattern, public_parameters)
    }

    /// Strips a plaintext pattern from the end of a `FheString`.
    ///
    /// Same as `strip_suffix` but with a plaintext pattern.
    ///
    /// # Example:
    /// ```
    /// let my_string_plain = "HELLO test test HELLO";
    /// let pattern_plain = "HELLO";
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let fhe_strip =
    ///     my_server_key.strip_suffix_clear(&my_string, &pattern_plain, &public_parameters);
    /// let (actual, flag) = FheStrip::decrypt(fhe_strip, &my_client_key);
    ///
    /// assert_eq!(actual, "HELLO test test ");
    /// assert_eq!(flag, 1u8);
    /// ```
    pub fn strip_suffix_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheStrip {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self.strip_suffix(string.clone(), &pattern, public_parameters)
    }

    fn comparison(
        &self,
        string: &FheString,
        other: &FheString,
        operation: Comparison,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);

        let mut min_length = usize::min(string.len(), other.len());
        let mut encountered_comparison = zero.clone();
        let mut has_flag_became_one = zero.clone();
        let two_five_five = FheAsciiChar::encrypt_trivial(255u8, public_parameters, &self.key);
        let mut ret = FheAsciiChar::encrypt_trivial(255u8, public_parameters, &self.key);

        // We clone since we need to potentially pad the strings
        let mut string_clone = string.clone();
        let mut other_clone = other.clone();

        // Edge case workaround, this happens if strings are unpadded
        if min_length == 0 {
            string_clone.push(zero.clone());
            other_clone.push(zero.clone());
            min_length = 1;
        }

        for i in 0..min_length {
            let comparison_result = match operation {
                Comparison::LessThan => string_clone[i].lt(&self.key, &other_clone[i]),
                Comparison::LessEqual => string_clone[i].le(&self.key, &other_clone[i]),
                Comparison::GreaterThan => string_clone[i].gt(&self.key, &other_clone[i]),
                Comparison::GreaterEqual => string_clone[i].ge(&self.key, &other_clone[i]),
            };

            let is_ne = string_clone[i].ne(&self.key, &other_clone[i]);

            encountered_comparison = encountered_comparison.bitor(&self.key, &is_ne); // skip when the prefix is common among strings

            let flag = encountered_comparison.bitand(
                &self.key,
                &has_flag_became_one.flip(&self.key, public_parameters),
            );
            has_flag_became_one = has_flag_became_one.bitor(&self.key, &flag); // this flag is required to only consider the first character we compare
            ret = flag.if_then_else(&self.key, &comparison_result, &ret)
        }

        // if ret = 255u8 it means that we never compared anything, which means the 2 strings are
        // equal
        let are_substrings_equal = ret.eq(&self.key, &two_five_five);

        let len1 = self.len(&string_clone, public_parameters);
        let len2 = self.len(&other_clone, public_parameters);

        let is_length_equal = len1.eq(&self.key, &len2);
        let is_length_greater_than = len1.gt(&self.key, &len2);
        let is_length_less_than = len1.lt(&self.key, &len2);

        let length_based_comparison = match operation {
            Comparison::GreaterEqual => is_length_equal.bitor(&self.key, &is_length_greater_than),
            Comparison::LessEqual => is_length_equal.bitor(&self.key, &is_length_less_than),
            Comparison::GreaterThan => is_length_greater_than,
            Comparison::LessThan => is_length_less_than,
        };

        // If we have 2 strings like so  "aaaa" and "aa"
        // They will appear equal as we are comparing only the first 2 elements of both
        // So to make sure they are actually equal we are also doing a length based
        // comparison at the end
        ret = are_substrings_equal.if_then_else(&self.key, &length_based_comparison, &ret);

        ret
    }

    /// Checks if the first `FheString` is less than the second `FheString`.
    ///
    /// # Arguments
    /// * `string`: &FheString - The first string to compare.
    /// * `other`: &FheString - The second string to compare.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheAsciiChar` - Encrypted 1 if the first string is less than the second, otherwise
    /// encrypted 0.
    ///
    /// # Example:
    /// ```
    /// let heistack1_plain = "hello test";
    /// let heistack2_plain = "hello test";
    ///
    /// let heistack1 = my_client_key.encrypt(
    ///     heistack1_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let heistack2 = my_client_key.encrypt(
    ///     heistack2_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.lt(&heistack1, &heistack2, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 0u8);
    /// ```
    pub fn lt(
        &self,
        string: &FheString,
        other: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        self.comparison(string, other, Comparison::LessThan, public_parameters)
    }

    /// Checks if the first `FheString` is less than or equal to the second `FheString`.
    ///
    /// Same as `lt` but checks for less than or equal to.
    ///
    /// # Example:
    /// ```
    /// let heistack1_plain = "hello test";
    /// let heistack2_plain = "hello test";
    ///
    /// let heistack1 = my_client_key.encrypt(
    ///     heistack1_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let heistack2 = my_client_key.encrypt(
    ///     heistack2_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.le(&heistack1, &heistack2, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn le(
        &self,
        string: &FheString,
        other: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        self.comparison(string, other, Comparison::LessEqual, public_parameters)
    }

    /// Checks if the first `FheString` is greater than the second `FheString`.
    ///
    /// Same as `lt` but checks for greater than.
    ///
    /// # Example:
    /// ```
    /// let heistack1_plain = "hello test";
    /// let heistack2_plain = "hello test";
    ///
    /// let heistack1 = my_client_key.encrypt(
    ///     heistack1_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let heistack2 = my_client_key.encrypt(
    ///     heistack2_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.gt(&heistack1, &heistack2, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 0u8);
    /// ```
    pub fn gt(
        &self,
        string: &FheString,
        other: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        self.comparison(string, other, Comparison::GreaterThan, public_parameters)
    }

    /// Checks if the first `FheString` is greater than or equal to the second `FheString`.
    ///
    /// Same as `lt` but checks for greater than or equal to.
    ///
    /// # Example:
    /// ```
    /// let heistack1_plain = "hello test";
    /// let heistack2_plain = "hello test";
    ///
    /// let heistack1 = my_client_key.encrypt(
    ///     heistack1_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let heistack2 = my_client_key.encrypt(
    ///     heistack2_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let res = my_server_key.ge(&heistack1, &heistack2, &public_parameters);
    /// let dec: u8 = my_client_key.decrypt_char(&res);
    ///
    /// assert_eq!(dec, 1u8);
    /// ```
    pub fn ge(
        &self,
        string: &FheString,
        other: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheAsciiChar {
        self.comparison(string, other, Comparison::GreaterEqual, public_parameters)
    }

    /// Replaces occurrences of a pattern in a given `FheString` with another pattern, up to `n`
    /// times.
    ///
    /// # Arguments
    /// * `string`: &FheString - The string in which replacements are to be made.
    /// * `from`: &Vec<FheAsciiChar> - The unpadded pattern to be replaced.
    /// * `to`: &Vec<FheAsciiChar> - The unpadded pattern to replace with.
    /// * `n`: FheAsciiChar - The encrypted maximum number of replacements.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheString` - The string with replacements made up to `n` times.
    /// Example:
    ///
    /// ```
    /// let my_string_plain = "hello abc abc test";
    /// let from_plain = "abc";
    /// let to_plain = "world";
    /// let n_plain = 1u8;
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let from = my_client_key.encrypt_no_padding(from_plain);
    /// let to = my_client_key.encrypt_no_padding(to_plain);
    /// let n = my_client_key.encrypt_char(n_plain);
    ///
    /// let my_new_string = my_server_key.replacen(&my_string, &from, &to, n, &public_parameters);
    /// let actual = my_client_key.decrypt(my_new_string);
    ///
    /// assert_eq!(actual, "hello world abc test");
    /// ```
    pub fn replacen(
        &self,
        string: &FheString,
        from: &Vec<FheAsciiChar>,
        to: &Vec<FheAsciiChar>,
        n: FheAsciiChar,
        public_parameters: &PublicParameters,
    ) -> FheString {
        if from.len() >= to.len() {
            Self::handle_longer_from(
                string.clone(),
                from.clone(),
                to.clone(),
                n,
                true,
                &self.key,
                public_parameters,
            )
        } else {
            Self::handle_shorter_from(
                string.clone(),
                from.clone(),
                to.clone(),
                n,
                true,
                &self.key,
                public_parameters,
            )
        }
    }

    /// Replaces occurrences of a plaintext pattern in a given `FheString` with another plaintext
    /// pattern, up to `n` times in plaintext.
    ///
    /// Same as `replacen` but with plaintext patterns and plaintext count.
    /// # Example:
    /// ```
    /// let my_string_plain = "hello abc abc test";
    /// let from_plain = "abc";
    /// let to_plain = "world";
    /// let n_plain = 1u8;
    ///
    /// let my_string = my_client_key.encrypt(
    ///     my_string_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    ///
    /// let my_new_string = my_server_key.replacen_clear(
    ///     &my_string,
    ///     &from_plain,
    ///     &to_plain,
    ///     n_plain,
    ///     &public_parameters,
    /// );
    /// let actual = my_client_key.decrypt(my_new_string);
    ///
    /// assert_eq!(actual, "hello world abc test");
    /// ```
    pub fn replacen_clear(
        &self,
        string: &FheString,
        from_clear: &str,
        to_clear: &str,
        n_clear: u8,
        public_parameters: &PublicParameters,
    ) -> FheString {
        let from = from_clear
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();

        let to = to_clear
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();

        let n = FheAsciiChar::encrypt_trivial(n_clear, public_parameters, &self.key);

        if from.len() >= to.len() {
            Self::handle_longer_from(
                string.clone(),
                from.clone(),
                to.clone(),
                n,
                true,
                &self.key,
                public_parameters,
            )
        } else {
            Self::handle_shorter_from(
                string.clone(),
                from.clone(),
                to.clone(),
                n,
                true,
                &self.key,
                public_parameters,
            )
        }
    }

    /// Concatenates two `FheString` instances into one.
    ///
    /// # Arguments
    /// * `string`: &FheString - The first string to concatenate.
    /// * `other`: &FheString - The second string to concatenate.
    /// * `public_parameters`: &PublicParameters - Public parameters for FHE operations.
    ///
    /// # Returns
    /// `FheString` - The concatenated result of the two strings.
    ///
    /// # Example:
    /// ```
    /// let my_string1_plain = "Hello, ";
    /// let my_string2_plain = "World!";
    ///
    /// let my_string1 = my_client_key.encrypt(
    ///     my_string1_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let my_string2 = my_client_key.encrypt(
    ///     my_string2_plain,
    ///     STRING_PADDING,
    ///     &public_parameters,
    ///     &my_server_key.key,
    /// );
    /// let my_string_upper = my_server_key.concatenate(&my_string1, &my_string2, &public_parameters);
    /// let actual = my_client_key.decrypt(my_string_upper);
    ///
    /// assert_eq!(actual, "Hello, World!");
    /// ```
    pub fn concatenate(
        &self,
        string: &FheString,
        other: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheString {
        let mut result = string.clone();
        let clone_other = other.clone();

        result.append(clone_other);
        utils::bubble_zeroes_right(result, &self.key, public_parameters)
    }
}
