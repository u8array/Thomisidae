use serde_json::{Value, json};
use std::io::{self, Write};

pub fn write_response_result(id: Value, result: Value) -> io::Result<()> {
    let resp = json!({ "jsonrpc": "2.0", "id": id, "result": result });
    let mut stdout = io::stdout();
    writeln!(stdout, "{}", resp)?;
    stdout.flush()
}

pub fn write_response_error(id: Value, code: i32, message: &str) -> io::Result<()> {
    let err = json!({ "code": code, "message": message });
    let resp = json!({ "jsonrpc": "2.0", "id": id, "error": err });
    let mut stdout = io::stdout();
    writeln!(stdout, "{}", resp)?;
    stdout.flush()
}
