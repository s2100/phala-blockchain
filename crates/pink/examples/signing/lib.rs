#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;


#[pink::contract(env=PinkEnvironment)]
mod signing {
    use super::pink;
    use pink::{PinkEnvironment, sign, verify, chain_extension::SigType};

    #[ink(storage)]
    pub struct Signing {}

    impl Signing {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn test(&self) {
            use hex_literal::hex;
            let privkey = hex!("78003ee90ff2544789399de83c60fa50b3b24ca86c7512d0680f64119207c80ab240b41344968b3e3a71a02c0e8b454658e00e9310f443935ecadbdd1674c683");
            let pubkey = hex!("ce786c340288b79a951c68f87da821d6c69abd1899dff695bda95e03f9c0b012");
            let message = b"hello world";
            let signature = sign!(message.to_vec(), privkey.to_vec(), SigType::Sr25519);
            let pass = verify!(message.to_vec(), pubkey.to_vec(), signature.to_vec(), SigType::Sr25519);
            assert_eq!(pass, true);
            let pass = verify!(b"Fake".to_vec(), pubkey.to_vec(), signature.to_vec(), SigType::Sr25519);
            assert_eq!(pass, false);
        }
    }
}
