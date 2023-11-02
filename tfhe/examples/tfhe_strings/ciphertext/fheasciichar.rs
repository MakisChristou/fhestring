use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXorAssign, Not, Sub, SubAssign,
};

use tfhe::{
    integer::{ciphertext::BaseRadixCiphertext, RadixClientKey},
    prelude::*,
    shortint::{server_key, Ciphertext, ServerKey},
};

#[derive(Clone)]
pub struct FheAsciiChar {
    pub inner: BaseRadixCiphertext<Ciphertext>,
}

impl FheAsciiChar {
    pub fn new(value: BaseRadixCiphertext<Ciphertext>) -> Self {
        FheAsciiChar { inner: value }
    }

    pub fn encrypt_trivial(
        value: u8,
        public_key: &tfhe::integer::PublicKey,
        num_blocks: usize,
    ) -> FheAsciiChar {
        FheAsciiChar::new(public_key.encrypt_radix(value as u64, num_blocks))
    }

    pub fn encrypt(value: u8, client_key: &RadixClientKey) -> FheAsciiChar {
        FheAsciiChar::new(client_key.encrypt(value as u64))
    }

    pub fn decrypt(value: &BaseRadixCiphertext<Ciphertext>, client_key: &RadixClientKey) -> u8 {
        client_key.decrypt::<u8>(value)
    }

    pub fn eq(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res = server_key.eq_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn ne(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res: BaseRadixCiphertext<Ciphertext> =
            server_key.ne_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn le(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res = server_key.le_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn lt(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res = server_key.lt_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn ge(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res: BaseRadixCiphertext<Ciphertext> =
            server_key.ge_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn gt(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res = server_key.gt_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn bitand(
        &self,
        server_key: &tfhe::integer::ServerKey,
        other: &FheAsciiChar,
    ) -> FheAsciiChar {
        let res = server_key.bitand_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn bitor(
        &self,
        server_key: &tfhe::integer::ServerKey,
        other: &FheAsciiChar,
    ) -> FheAsciiChar {
        let res = server_key.bitor_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn bitxor(
        &self,
        server_key: &tfhe::integer::ServerKey,
        other: &FheAsciiChar,
    ) -> FheAsciiChar {
        let res = server_key.bitxor_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn bitnot(&self, server_key: &tfhe::integer::ServerKey) -> FheAsciiChar {
        let res = server_key.bitnot_parallelized(&self.inner);
        FheAsciiChar::new(res)
    }

    pub fn sub(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res = server_key.sub_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn add(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res = server_key.add_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn mul(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res = server_key.mul_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn div(&self, server_key: &tfhe::integer::ServerKey, other: &FheAsciiChar) -> FheAsciiChar {
        let res = server_key.div_parallelized(&self.inner, &other.inner);
        FheAsciiChar::new(res)
    }

    pub fn if_then_else(
        &self,
        server_key: &tfhe::integer::ServerKey,
        true_value: &FheAsciiChar,
        false_value: &FheAsciiChar,
    ) -> FheAsciiChar {
        let res = server_key.if_then_else_parallelized(
            &self.inner,
            &true_value.inner,
            &false_value.inner,
        );
        FheAsciiChar::new(res)
    }

    // pub fn is_whitespace(&self) -> FheAsciiChar {
    //     let space = FheAsciiChar::encrypt_trivial(0x20u8); // Space
    //     let tab = FheAsciiChar::encrypt_trivial(0x09u8); // Horizontal Tab
    //     let newline = FheAsciiChar::encrypt_trivial(0x0Au8); // Newline
    //     let vertical_tab = FheAsciiChar::encrypt_trivial(0x0Bu8); // Vertical Tab
    //     let form_feed = FheAsciiChar::encrypt_trivial(0x0Cu8); // Form Feed
    //     let carriage_return = FheAsciiChar::encrypt_trivial(0x0Du8); // Carriage Return

    //     self.eq(&space)
    //         | self.eq(&tab)
    //         | self.eq(&newline)
    //         | self.eq(&vertical_tab)
    //         | self.eq(&form_feed)
    //         | self.eq(&carriage_return)
    // }

    pub fn is_uppercase(
        &self,
        server_key: &tfhe::integer::ServerKey,
        public_key: &tfhe::integer::PublicKey,
        num_blocks: usize,
    ) -> FheAsciiChar {
        let uppercase_a = FheAsciiChar::encrypt_trivial(0x41u8, public_key, num_blocks); // 'A'
        let uppercase_z = FheAsciiChar::encrypt_trivial(0x5Au8, public_key, num_blocks); // 'Z'

        let res1 = self.ge(server_key, &uppercase_a);
        let res2 = self.le(server_key, &uppercase_z);

        res1.bitand(server_key, &res2)
    }

    pub fn is_lowercase(
        &self,
        server_key: &tfhe::integer::ServerKey,
        public_key: &tfhe::integer::PublicKey,
        num_blocks: usize,
    ) -> FheAsciiChar {
        let lowercase_a = FheAsciiChar::encrypt_trivial(0x61u8, public_key, num_blocks); // 'a'
        let lowercase_z = FheAsciiChar::encrypt_trivial(0x7Au8, public_key, num_blocks); // 'z'

        let res1 = self.ge(server_key, &lowercase_a);
        let res2 = self.le(server_key, &lowercase_z);

        res1.bitand(server_key, &res2)
    }

    // pub fn is_alphabetic(&self) -> FheAsciiChar {
    //     let is_uppercase = self.is_uppercase();
    //     let is_lowercase = self.is_lowercase();

    //     is_uppercase | is_lowercase
    // }

    // pub fn is_number(&self) -> FheAsciiChar {
    //     let digit_0 = FheAsciiChar::encrypt_trivial(0x30u8); // '0'
    //     let digit_9 = FheAsciiChar::encrypt_trivial(0x39u8); // '9'

    //     self.ge(&digit_0) & self.le(&digit_9)
    // }

    // pub fn is_alphanumeric(&self) -> FheAsciiChar {
    //     let is_alphabetic = self.is_alphabetic();
    //     let is_number = self.is_number();

    //     is_alphabetic | is_number
    // }

    // // Input must be either 0 or 1
    pub fn flip(
        &self,
        server_key: &tfhe::integer::ServerKey,
        public_key: &tfhe::integer::PublicKey,
        num_blocks: usize,
    ) -> FheAsciiChar {
        let one = FheAsciiChar::encrypt_trivial(1u8, public_key, num_blocks);
        one.sub(server_key, self)
    }
}

// // Implementing Add
// impl Add for FheAsciiChar {
//     type Output = Self;

//     fn add(self, other: Self) -> Self {
//         Self::new(self.inner + other.inner)
//     }
// }

// // Implementing Add for references
// impl<'a, 'b> Add<&'b FheAsciiChar> for &'a FheAsciiChar {
//     type Output = FheAsciiChar;

//     fn add(self, other: &'b FheAsciiChar) -> FheAsciiChar {
//         FheAsciiChar::new(&self.inner + &other.inner)
//     }
// }

// // Implementing Sub
// impl Sub for FheAsciiChar {
//     type Output = Self;

//     fn sub(self, other: Self) -> Self {
//         Self::new(self.inner - other.inner)
//     }
// }

// // Implementing Sub for references
// impl<'a, 'b> Sub<&'b FheAsciiChar> for &'a FheAsciiChar {
//     type Output = FheAsciiChar;

//     fn sub(self, other: &'b FheAsciiChar) -> FheAsciiChar {
//         FheAsciiChar::new(&self.inner - &other.inner)
//     }
// }

// // Implementing Bitwise OR (|) for logical OR
// impl BitOr for FheAsciiChar {
//     type Output = Self;

//     fn bitor(self, other: Self) -> Self {
//         Self::new(self.inner | other.inner)
//     }
// }

// // Implementing Bitwise AND (&) for logical AND
// impl BitAnd for FheAsciiChar {
//     type Output = Self;

//     fn bitand(self, other: Self) -> Self {
//         Self::new(self.inner & other.inner)
//     }
// }

// // Implementing Bitwise NOT (!) for logical NOT
// impl Not for FheAsciiChar {
//     type Output = Self;

//     fn not(self) -> Self {
//         Self::new(!self.inner)
//     }
// }

// // Implementing Bitwise OR Assign (|=)
// impl BitOrAssign for FheAsciiChar {
//     fn bitor_assign(&mut self, other: Self) {
//         self.inner |= other.inner;
//     }
// }

// // Implementing Bitwise AND Assign (&=)
// impl BitAndAssign for FheAsciiChar {
//     fn bitand_assign(&mut self, other: Self) {
//         self.inner &= other.inner;
//     }
// }

// // Implementing Bitwise XOR Assign (^=)
// impl BitXorAssign for FheAsciiChar {
//     fn bitxor_assign(&mut self, other: Self) {
//         self.inner ^= other.inner;
//     }
// }

// // Implementing Add Assign (+=)
// impl AddAssign for FheAsciiChar {
//     fn add_assign(&mut self, other: Self) {
//         self.inner += other.inner;
//     }
// }

// // Implementing Subtract Assign (-=)
// impl SubAssign for FheAsciiChar {
//     fn sub_assign(&mut self, other: Self) {
//         self.inner -= other.inner;
//     }
// }
