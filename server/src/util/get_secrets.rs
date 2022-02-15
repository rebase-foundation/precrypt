use std::env;
use std::env::VarError;

const ORION_SECRET: &str = "ORION_SECRET";
const WEB3_TOKEN: &str = "WEB3s";

pub fn get_secrets() -> core::result::Result<(String, String), VarError> {
   let orion_string = env::var(ORION_SECRET);
   let web3_token = env::var(WEB3_TOKEN);

   if orion_string.is_err() {
      return Err(orion_string.unwrap_err());
   } 
   if web3_token.is_err() {
      return Err(web3_token.unwrap_err());
   }

   return Ok((orion_string.unwrap(), web3_token.unwrap()));
}