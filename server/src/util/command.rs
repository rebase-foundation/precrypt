use std::process::Command;

#[derive(Debug, Clone)]
struct CommandError;

pub fn run_command(cmd: String) -> std::io::Result<std::process::Output> {
   let result = Command::new("sh")
      .arg("-c")
      .arg(cmd)
      .output()?;
   if !result.stderr.is_empty() {
      let error = std::str::from_utf8(&result.stderr).unwrap();
      return Err(std::io::Error::new(std::io::ErrorKind::Other, error));
   }
   return Ok(result);
}