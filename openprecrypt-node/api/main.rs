use http::StatusCode;
use vercel_lambda::{lambda, error::VercelError, IntoResponse, Request, Response};

fn get_response(req: Request) -> Result<serde_json::Value, &'static str> {
    return json!({
        data: "hello"
    });
    // let body: Value = match req.body() {
    //     now_lambda::Body::Binary(data) => {
    //         serde_json::from_slice(data).map_err(|_| "request body is not valid json")?
    //     }
    //     _ => return Err("Request body is not in binary format"),
    // };
}

fn handler(req: Request) -> Result<impl IntoResponse, NowError> {
    match get_response(req) {
        Ok(res) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(res.to_string())
            .expect("Something happened")),

        Err(e) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(e.into())
            .expect("Something happened")),
    }
}

// Start the runtime with the handler
fn main() -> Result<(), Box<dyn Error>> {
    Ok(lambda!(handler))
}
