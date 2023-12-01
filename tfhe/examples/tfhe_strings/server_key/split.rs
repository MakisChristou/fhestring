use crate::ciphertext::fheasciichar::FheAsciiChar;
use crate::ciphertext::fhesplit::FheSplit;
use crate::ciphertext::fhestring::FheString;
use crate::ciphertext::public_parameters::PublicParameters;
use crate::utils;

use super::MyServerKey;

impl MyServerKey {
    fn _rsplit(
        &self,
        string: &FheString,
        pattern: Vec<FheAsciiChar>,
        is_inclusive: bool,
        is_terminator: bool,
        n: Option<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let max_buffer_size = string.len(); // when a single buffer holds the whole input
        let max_no_buffers = max_buffer_size; // when all buffers hold an empty value

        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let mut current_copy_buffer = zero.clone();
        let mut stop_counter_increment = zero.clone();
        let mut result = vec![vec![zero.clone(); max_buffer_size]; max_no_buffers];
        let mut global_pattern_found = one.clone();

        // Edge case flag, if n = 0 we ever copy anything
        let mut allow_copying = zero.clone();

        if n.is_some() {
            let n_value = n.clone().unwrap();
            allow_copying = n_value.ne(&self.key, &zero);
        }

        for i in (0..(string.len())).rev() {
            // Copy ith character to the appropriate buffer
            for (j, result_item) in result.iter_mut().enumerate().take(max_no_buffers) {
                let enc_j = FheAsciiChar::encrypt_trivial(j as u8, public_parameters, &self.key);
                let mut copy_flag = enc_j.eq(&self.key, &current_copy_buffer);

                // Edge case, if n = 0 we ever copy anything
                if n.is_some() {
                    copy_flag = copy_flag.bitand(&self.key, &allow_copying);
                }

                result_item[i] = copy_flag.if_then_else(&self.key, &string[i], &result_item[i]);
            }

            let mut pattern_found = one.clone();
            // Avoid index out of bounds error
            if i + pattern.len() >= string.len() {
                pattern_found = zero.clone();
            } else {
                for (j, pattern_char) in pattern.iter().enumerate() {
                    let eql = string[i + j].eq(&self.key, pattern_char);
                    pattern_found = pattern_found.bitand(&self.key, &eql);
                }
            }

            global_pattern_found = global_pattern_found.bitand(&self.key, &pattern_found);

            // If its splitn stop after n splits
            match &n {
                None => {
                    // Here we know if the pattern is found for position i
                    // If its found we need to switch from copying to old buffer and start copying
                    // to new one
                    current_copy_buffer = pattern_found.if_then_else(
                        &self.key,
                        &current_copy_buffer.add(&self.key, &one),
                        &current_copy_buffer,
                    );
                }
                Some(max_splits) => {
                    stop_counter_increment = stop_counter_increment.bitor(
                        &self.key,
                        &current_copy_buffer.eq(&self.key, &max_splits.sub(&self.key, &one)),
                    );

                    // Here we know if the pattern is found for position i
                    // If its found we need to switch from copying to old buffer and start copying
                    // to new one
                    current_copy_buffer = (pattern_found.bitand(
                        &self.key,
                        &stop_counter_increment.flip(&self.key, public_parameters),
                    ))
                    .if_then_else(
                        &self.key,
                        &current_copy_buffer.add(&self.key, &one),
                        &current_copy_buffer,
                    );
                }
            };
        }

        match &n {
            Some(max_splits) => {
                let to: Vec<FheAsciiChar> = "\0"
                    .repeat(pattern.len())
                    .as_bytes()
                    .iter()
                    .map(|b| FheAsciiChar::encrypt_trivial(*b, public_parameters, &self.key))
                    .collect();
                let mut stop_replacing_pattern = zero.clone();

                for (i, result_buffer) in result.iter_mut().enumerate().take(max_no_buffers) {
                    let enc_i =
                        FheAsciiChar::encrypt_trivial(i as u8, public_parameters, &self.key);
                    stop_replacing_pattern = stop_replacing_pattern.bitor(
                        &self.key,
                        &max_splits.eq(&self.key, &enc_i.add(&self.key, &one)),
                    );

                    let current_string =
                        FheString::from_vec(result_buffer.clone(), public_parameters, &self.key);
                    let current_string =
                        utils::bubble_zeroes_left(current_string, &self.key, public_parameters);
                    let replacement_string =
                        self.replace(&current_string, &pattern, &to, public_parameters);

                    // Don't remove pattern from (n-1)th buffer
                    for (j, result_buffer_char) in
                        result_buffer.iter_mut().enumerate().take(max_buffer_size)
                    {
                        *result_buffer_char = stop_replacing_pattern.if_then_else(
                            &self.key,
                            &current_string[j],
                            &replacement_string[j],
                        );
                    }
                }
            }
            None => {
                if !is_inclusive {
                    let to: Vec<FheAsciiChar> = "\0"
                        .repeat(pattern.len())
                        .as_bytes()
                        .iter()
                        .map(|b| FheAsciiChar::encrypt_trivial(*b, public_parameters, &self.key))
                        .collect();

                    // Since the pattern is also copied at the end of each buffer go through them
                    // and delete it
                    for result_buffer in result.iter_mut().take(max_no_buffers) {
                        let current_string = FheString::from_vec(
                            result_buffer.clone(),
                            public_parameters,
                            &self.key,
                        );
                        let replacement_string =
                            self.replace(&current_string, &pattern, &to, public_parameters);
                        *result_buffer = replacement_string.get_bytes();
                    }
                } else {
                    for result_buffer in result.iter_mut().take(max_no_buffers) {
                        let new_buf = utils::bubble_zeroes_left(
                            FheString::from_vec(
                                result_buffer.clone(),
                                public_parameters,
                                &self.key,
                            ),
                            &self.key,
                            public_parameters,
                        );
                        *result_buffer = new_buf.get_bytes();
                    }
                }

                // Zero out the last populated buffer if it starts with the pattern
                if is_terminator {
                    let mut non_zero_buffer_found = zero.clone();
                    for i in (0..max_no_buffers).rev() {
                        let mut is_buff_zero = one.clone();

                        for j in 0..max_buffer_size {
                            is_buff_zero =
                                is_buff_zero.bitand(&self.key, &result[i][j].eq(&self.key, &zero));
                        }

                        // Here we know if the current buffer is non-empty
                        // Now we have to check if it starts with the pattern
                        let starts_with_pattern = self.starts_with(
                            &FheString::from_vec(result[i].clone(), public_parameters, &self.key),
                            &pattern,
                            public_parameters,
                        );
                        let should_delete =
                            starts_with_pattern.bitand(&self.key, &is_buff_zero).bitand(
                                &self.key,
                                &non_zero_buffer_found.flip(&self.key, public_parameters),
                            );

                        for j in 0..max_buffer_size {
                            result[i][j] =
                                should_delete.if_then_else(&self.key, &zero, &result[i][j])
                        }
                        non_zero_buffer_found = non_zero_buffer_found
                            .bitor(&self.key, &is_buff_zero.flip(&self.key, public_parameters));
                    }
                }
            }
        }

        FheSplit::new(result, global_pattern_found, public_parameters, &self.key)
    }

    pub fn rsplit(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        self._rsplit(
            string,
            pattern.to_owned(),
            false,
            false,
            None,
            public_parameters,
        )
    }

    pub fn rsplit_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self.rsplit(string, &pattern, public_parameters)
    }

    pub fn rsplitn(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        n: FheAsciiChar,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        self._rsplit(
            string,
            pattern.to_owned(),
            false,
            false,
            Some(n),
            public_parameters,
        )
    }

    pub fn rsplitn_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        clear_n: usize,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        let n = FheAsciiChar::encrypt_trivial(clear_n as u8, public_parameters, &self.key);
        self._rsplit(string, pattern, false, false, Some(n), public_parameters)
    }

    pub fn rsplit_once(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let n = FheAsciiChar::encrypt_trivial(2u8, public_parameters, &self.key);
        self._rsplit(
            string,
            pattern.to_owned(),
            false,
            false,
            Some(n),
            public_parameters,
        )
    }

    pub fn rsplit_once_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        let n = FheAsciiChar::encrypt_trivial(2u8, public_parameters, &self.key);
        self._rsplit(string, pattern, false, false, Some(n), public_parameters)
    }

    pub fn rsplit_terminator(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        self._rsplit(
            string,
            pattern.to_owned(),
            false,
            true,
            None,
            public_parameters,
        )
    }

    pub fn rsplit_terminator_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self._rsplit(string, pattern, false, true, None, public_parameters)
    }

    fn _split(
        &self,
        string: &FheString,
        pattern: Vec<FheAsciiChar>,
        is_inclusive: bool,
        is_terminator: bool,
        n: Option<FheAsciiChar>,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let max_buffer_size = string.len(); // when a single buffer holds the whole input
        let max_no_buffers = max_buffer_size; // when all buffers hold an empty value

        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let mut current_copy_buffer = zero.clone();
        let mut stop_counter_increment = zero.clone();
        let mut result = vec![vec![zero.clone(); max_buffer_size]; max_no_buffers];
        let mut global_pattern_found = one.clone();

        // Edge case flag, if n = 0 we ever copy anything
        let mut allow_copying = zero.clone();

        if n.is_some() {
            let n_value = n.clone().unwrap();
            allow_copying = n_value.ne(&self.key, &zero);
        }

        // Edge case flag, explanation below
        let mut should_skip_first_buffer = zero.clone();

        // Handle edge case when 1 < n <= string.len() and pattern is empty
        // In this case we should leave an empty buffer effectively skipping the first one
        // Example1:  "eeeeee".rsplitn(2, "") --> ["", "eeeeee"]
        // Example2:  "eeeeee".rsplitn(3, "") --> ["", "e", "eeeee"]
        if pattern.is_empty() && n.is_some() {
            let n_value = n.clone().unwrap();
            let enc_len = self.len(string, public_parameters);

            should_skip_first_buffer = n_value
                .gt(&self.key, &one)
                .bitand(&self.key, &n_value.le(&self.key, &enc_len));

            current_copy_buffer = should_skip_first_buffer.if_then_else(
                &self.key,
                &FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key),
                &current_copy_buffer,
            );
        }

        for i in 0..(string.len()) {
            // Copy ith character to the appropriate buffer
            for (j, result_buffer) in result.iter_mut().enumerate().take(max_no_buffers) {
                let enc_j = FheAsciiChar::encrypt_trivial(j as u8, public_parameters, &self.key);
                let mut copy_flag = enc_j.eq(&self.key, &current_copy_buffer);

                // Edge case, if n = 0 we ever copy anything
                if n.is_some() {
                    copy_flag = copy_flag.bitand(&self.key, &allow_copying);
                }

                result_buffer[i] = copy_flag.if_then_else(&self.key, &string[i], &result_buffer[i]);
            }

            let mut pattern_found = one.clone();
            // To avoid underflow
            if (i as i64) < (pattern.len() as i64) - 1 {
                pattern_found = zero.clone();
            } else {
                for (j, pattern_char) in pattern.iter().enumerate() {
                    let string_index = i - pattern.len() + 1 + j;
                    let eql = string[string_index].eq(&self.key, pattern_char);
                    pattern_found = pattern_found.bitand(&self.key, &eql);
                }
            }

            global_pattern_found = global_pattern_found.bitand(&self.key, &pattern_found);

            // If its splitn stop after n splits
            match &n {
                None => {
                    // Here we know if the pattern is found for position i
                    // If its found we need to switch from copying to old buffer and start copying
                    // to new one
                    current_copy_buffer = pattern_found.if_then_else(
                        &self.key,
                        &current_copy_buffer.add(&self.key, &one),
                        &current_copy_buffer,
                    );
                }
                Some(max_splits) => {
                    stop_counter_increment = stop_counter_increment.bitor(
                        &self.key,
                        &current_copy_buffer.eq(&self.key, &max_splits.sub(&self.key, &one)),
                    );

                    // Here we know if the pattern is found for position i
                    // If its found we need to switch from copying to old buffer and start copying
                    // to new one
                    current_copy_buffer = (pattern_found.bitand(
                        &self.key,
                        &stop_counter_increment.flip(&self.key, public_parameters),
                    ))
                    .if_then_else(
                        &self.key,
                        &current_copy_buffer.add(&self.key, &one),
                        &current_copy_buffer,
                    );
                }
            };
        }

        match &n {
            Some(max_splits) => {
                let to: Vec<FheAsciiChar> = "\0"
                    .repeat(pattern.len())
                    .as_bytes()
                    .iter()
                    .map(|b| FheAsciiChar::encrypt_trivial(*b, public_parameters, &self.key))
                    .collect();
                let mut stop_replacing_pattern = zero.clone();

                for (i, result_buffer) in result.iter_mut().enumerate().take(max_no_buffers) {
                    // Check if we have reached the max allowed splits
                    let enc_i =
                        FheAsciiChar::encrypt_trivial(i as u8, public_parameters, &self.key);
                    stop_replacing_pattern = stop_replacing_pattern.bitor(
                        &self.key,
                        &max_splits.eq(&self.key, &enc_i.add(&self.key, &one)),
                    );

                    let current_string =
                        FheString::from_vec(result_buffer.clone(), public_parameters, &self.key);
                    let current_string =
                        utils::bubble_zeroes_left(current_string, &self.key, public_parameters);
                    let replacement_string =
                        self.replace(&current_string, &pattern, &to, public_parameters);

                    // Don't remove pattern from (n-1)th buffer
                    for (j, result_buffer_char) in
                        result_buffer.iter_mut().enumerate().take(max_buffer_size)
                    {
                        *result_buffer_char = stop_replacing_pattern.if_then_else(
                            &self.key,
                            &current_string[j],
                            &replacement_string[j],
                        );
                    }
                }
            }
            None => {
                // If its not inclusive we have to remove the pattern
                // We do that by replacing it with zeroes and bubble them to the end
                if !is_inclusive {
                    let to: Vec<FheAsciiChar> = "\0"
                        .repeat(pattern.len())
                        .as_bytes()
                        .iter()
                        .map(|b| FheAsciiChar::encrypt_trivial(*b, public_parameters, &self.key))
                        .collect();

                    // Since the pattern is also copied at the end of each buffer go through them
                    // and delete it
                    for result_buffer in result.iter_mut().take(max_no_buffers) {
                        let current_string = FheString::from_vec(
                            result_buffer.clone(),
                            public_parameters,
                            &self.key,
                        );
                        let replacement_string =
                            self.replace(&current_string, &pattern, &to, public_parameters);
                        *result_buffer = replacement_string.get_bytes();
                    }
                } else {
                    for result_buffer in result.iter_mut().take(max_no_buffers) {
                        let new_buf = utils::bubble_zeroes_left(
                            FheString::from_vec(
                                result_buffer.clone(),
                                public_parameters,
                                &self.key,
                            ),
                            &self.key,
                            public_parameters,
                        );
                        *result_buffer = new_buf.get_bytes();
                    }
                }

                // Zero out the last populated buffer if it starts with the pattern
                if is_terminator {
                    let mut non_zero_buffer_found = zero.clone();
                    for i in (0..max_no_buffers).rev() {
                        let mut is_buff_zero = one.clone();

                        for j in 0..max_buffer_size {
                            is_buff_zero =
                                is_buff_zero.bitand(&self.key, &result[i][j].eq(&self.key, &zero));
                        }

                        // Here we know if the current buffer is non-empty
                        // Now we have to check if it starts with the pattern
                        let starts_with_pattern = self.starts_with(
                            &FheString::from_vec(result[i].clone(), public_parameters, &self.key),
                            &pattern,
                            public_parameters,
                        );
                        let should_delete =
                            starts_with_pattern.bitand(&self.key, &is_buff_zero).bitand(
                                &self.key,
                                &non_zero_buffer_found.flip(&self.key, public_parameters),
                            );

                        for j in 0..max_buffer_size {
                            result[i][j] =
                                should_delete.if_then_else(&self.key, &zero, &result[i][j]);
                        }

                        non_zero_buffer_found = non_zero_buffer_found
                            .bitor(&self.key, &is_buff_zero.flip(&self.key, public_parameters));
                    }
                }
            }
        }

        FheSplit::new(result, global_pattern_found, public_parameters, &self.key)
    }

    pub fn split(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        self._split(
            string,
            pattern.to_owned(),
            false,
            false,
            None,
            public_parameters,
        )
    }

    pub fn split_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self.split(string, &pattern, public_parameters)
    }

    pub fn split_inclusive(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        self._split(
            string,
            pattern.to_owned(),
            true,
            false,
            None,
            public_parameters,
        )
    }

    pub fn split_inclusive_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self.split_inclusive(string, &pattern, public_parameters)
    }

    pub fn split_terminator(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        self._split(
            string,
            pattern.to_owned(),
            false,
            true,
            None,
            public_parameters,
        )
    }

    pub fn split_terminator_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        self._split(
            string,
            pattern.to_owned(),
            false,
            true,
            None,
            public_parameters,
        )
    }

    pub fn split_ascii_whitespace(
        &self,
        string: &FheString,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let max_buffer_size = string.len(); // when a single buffer holds the whole input
        let max_no_buffers = max_buffer_size; // when all buffers hold an empty value

        let zero = FheAsciiChar::encrypt_trivial(0u8, public_parameters, &self.key);
        let one = FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let mut current_copy_buffer = zero.clone();
        let mut result = vec![vec![zero.clone(); max_buffer_size]; max_no_buffers];
        let mut previous_was_whitespace =
            FheAsciiChar::encrypt_trivial(1u8, public_parameters, &self.key);
        let mut global_pattern_found = one.clone();

        for i in 0..(string.len()) {
            let pattern_found = string[i].is_whitespace(&self.key, public_parameters);
            global_pattern_found = global_pattern_found.bitand(&self.key, &pattern_found);

            let should_increment_buffer = pattern_found.bitand(
                &self.key,
                &previous_was_whitespace.flip(&self.key, public_parameters),
            );

            // Here we know if the pattern is found for position i
            // If its found we need to switch from copying to old buffer and start copying
            // to new one
            current_copy_buffer = should_increment_buffer.if_then_else(
                &self.key,
                &current_copy_buffer.add(&self.key, &one),
                &current_copy_buffer,
            );

            // Copy ith character to the appropriate buffer
            for (j, result_buffer) in result.iter_mut().enumerate().take(max_no_buffers) {
                let enc_j = FheAsciiChar::encrypt_trivial(j as u8, public_parameters, &self.key);
                let mut copy_flag = enc_j.eq(&self.key, &current_copy_buffer);
                copy_flag = copy_flag.bitand(
                    &self.key,
                    &string[i]
                        .is_whitespace(&self.key, public_parameters)
                        .flip(&self.key, public_parameters),
                ); // copy if its not whitespace
                result_buffer[i] = copy_flag.if_then_else(&self.key, &string[i], &result_buffer[i]);
            }

            previous_was_whitespace = pattern_found;
        }

        // Replace whitespace with \0
        for result_buffer in result.iter_mut().take(max_no_buffers) {
            for result_buffer_char in result_buffer.iter_mut().take(max_buffer_size) {
                let replace_with_zero =
                    result_buffer_char.is_whitespace(&self.key, public_parameters);
                *result_buffer_char =
                    replace_with_zero.if_then_else(&self.key, &zero, result_buffer_char);
            }
        }

        for result_buffer in result.iter_mut().take(max_no_buffers) {
            let new_buf = utils::bubble_zeroes_left(
                FheString::from_vec(result_buffer.clone(), public_parameters, &self.key),
                &self.key,
                public_parameters,
            );
            *result_buffer = new_buf.get_bytes();
        }

        FheSplit::new(result, global_pattern_found, public_parameters, &self.key)
    }

    pub fn splitn(
        &self,
        string: &FheString,
        pattern: &[FheAsciiChar],
        n: FheAsciiChar,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        self._split(
            string,
            pattern.to_owned(),
            false,
            false,
            Some(n),
            public_parameters,
        )
    }

    pub fn splitn_clear(
        &self,
        string: &FheString,
        clear_pattern: &str,
        clear_n: usize,
        public_parameters: &PublicParameters,
    ) -> FheSplit {
        let pattern = clear_pattern
            .bytes()
            .map(|b| FheAsciiChar::encrypt_trivial(b, public_parameters, &self.key))
            .collect::<Vec<FheAsciiChar>>();
        let n = FheAsciiChar::encrypt_trivial(clear_n as u8, public_parameters, &self.key);
        self._split(string, pattern, false, false, Some(n), public_parameters)
    }
}
