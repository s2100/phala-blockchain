use ink_lang as ink;
use alloc::vec::Vec;

pub use http_request::{HttpRequest, HttpResponse};
pub use signing::{SignArgs, VerifyArgs, SigType};

mod http_request;
mod signing;

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ErrorCode {}

impl ink_env::chain_extension::FromStatusCode for ErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            _ => panic!("encountered unknown status code"),
        }
    }
}

/// Extensions for the ink runtime defined by fat contract.
#[ink::chain_extension]
pub trait PinkExt {
    type ErrorCode = ErrorCode;

    // func_id refer to https://github.com/patractlabs/PIPs/blob/main/PIPs/pip-100.md
    #[ink(extension = 0xff000001, handle_status = false, returns_result = false)]
    fn http_request(request: HttpRequest) -> HttpResponse;

    #[ink(extension = 0xff000002, handle_status = false, returns_result = false)]
    fn sign(args: SignArgs) -> Vec<u8>;

    #[ink(extension = 0xff000003, handle_status = false, returns_result = false)]
    fn verify(args: VerifyArgs) -> bool;
}
