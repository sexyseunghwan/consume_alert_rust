use crate::common::*;

#[doc = "Function to globally initialize the 'CONSUME_DETAIL' variable"]
pub static CONSUME_DETAIL: once_lazy<String> = once_lazy::new(|| {
    dotenv().ok();
    env::var("CONSUME_DETAIL").expect("[ENV file read Error] 'CONSUME_DETAIL' must be set")
});

#[doc = "Function to globally initialize the 'CONSUME_TYPE' variable"]
pub static CONSUME_TYPE: once_lazy<String> = once_lazy::new(|| {
    dotenv().ok();
    env::var("CONSUME_TYPE").expect("[ENV file read Error] 'CONSUME_TYPE' must be set")
});
