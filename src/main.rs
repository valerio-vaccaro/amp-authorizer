#![feature(proc_macro_hygiene, decl_macro)]
use colored::*;

#[macro_use] extern crate rocket;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[cfg(test)] mod test;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Input {
    amount: u64,
    gaid: String,
    investor: Option<u8>,
    is_treasury: bool,
    registered_user: Option<u8>,
    vin: u8,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Output {
    amount: u64,
    gaid: String,
    investor: Option<u8>,
    is_treasury: bool,
    registered_user: Option<u8>,
    vout: u8,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Request {
    asset_id: String,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    tx_hex: String,
    uuid: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Message {
    request: Request,
    server_result: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AuthRequest {
    message:  Message,
    signature: String,
}

#[derive(Debug)]
enum AuthError {
    JsonParsingError,
    WrongSignature,
    UnauthorizedAsset,
    UnauthorizedInput,
    UnauthorizedOutput,
    UnauthorizedAmount,
    UnauthorizedAmountUnderMin,
    UnauthorizedAmountOverMax,
}

#[get("/")]
fn main_api() -> String {
    format!("error")
}

#[get("/issuerauthorizer")]
fn authorizer_api_get() -> String {
    format!("error")
}

#[post("/issuerauthorizer", data = "<request_message>")]
fn authorizer_api(request_message: String) -> String {
    let message = validate(&request_message);
    println!("{}", message);
    if message == "ok" {
        let result = json!({
            "result": true,
            "error": "",
        });
        format!("{}", result.to_string())
    } else {
        let result = json!({
            "result": false,
            "error": message,
        });
        format!("{}", result.to_string())
    }
}

fn parse_message(data: &str) -> Result<AuthRequest, AuthError> {
    let request: Result<AuthRequest, serde_json::Error> = serde_json::from_str(data);
    match request {
        Ok(res) => Ok(res),
        Err(error) => Err(AuthError::JsonParsingError),
    }
}

fn validate_signature(request: &AuthRequest) -> Result<(), AuthError>{
    Ok(())
    //Err(AuthError::WrongSignature)
}

fn validate_asset_id(request: &AuthRequest, auth_asset_id: Vec<&str>) -> Result<(), AuthError>{
    let asset_id = &request.message.request.asset_id;
    if auth_asset_id.contains(&asset_id.as_str()) {
        Ok(())
    } else {
        Err(AuthError::UnauthorizedAsset)
    }
}

fn validate_inputs(request: &AuthRequest, auth_gaid_in: Vec<&str>) -> Result<(), AuthError>{
    let inputs = &request.message.request.inputs;
    for i in 0..inputs.len() {
        if ! auth_gaid_in.contains(&inputs[i].gaid.as_str()){
            return Err(AuthError::UnauthorizedInput);
        }
    }
    Ok(())
}

fn validate_outputs(request: &AuthRequest, auth_gaid_out: Vec<&str>, auth_allow_changes: bool) -> Result<(), AuthError>{
    let inputs = &request.message.request.inputs;
    let outputs = &request.message.request.outputs;
    let mut auth_gaid_out_local = auth_gaid_out;
    if auth_allow_changes {
        for i in 0..inputs.len() {
            auth_gaid_out_local.push(&inputs[i].gaid.as_str());
        }
    }
    for i in 0..outputs.len() {
        if ! auth_gaid_out_local.contains(&outputs[i].gaid.as_str()){
            return Err(AuthError::UnauthorizedOutput);
        }
    }
    Ok(())
}

fn validate_amounts(request: &AuthRequest, auth_min_amount: u64, auth_max_amount: u64) -> Result<(), AuthError>{
    let inputs = &request.message.request.inputs;
    let outputs = &request.message.request.outputs;
    let mut total_in = 0;
    let mut total_out = 0;
    for i in 0..inputs.len() {
        total_in = total_in + inputs[i].amount;
    }
    for i in 0..outputs.len() {
        total_out = total_out + outputs[i].amount;
        if outputs[i].amount < auth_min_amount {
            return Err(AuthError::UnauthorizedAmountUnderMin);
        }
        if outputs[i].amount > auth_max_amount {
            return Err(AuthError::UnauthorizedAmountOverMax);
        }

    }
    if total_in != total_out {
        return Err(AuthError::UnauthorizedAmount);
    }
    Ok(())
}

fn validate(data: &str) -> &str {
    // validation constants
    let auth_asset_id = vec!["6129504dafd3924f1cd18087da1e907e4d8813529b489d0883e82de79a6b0bad"];
    let auth_gaid_in = vec!["GA2pcpx9Yw1cDMGiSENKd81TiqD3DN"];
    let auth_gaid_out = vec!["GA2nbvvbayNahDUB7MBpam5jCnfETJ"];
    let auth_allow_changes: bool = true;
    let auth_min_amount: u64 = 0;
    let auth_max_amount: u64 = 2100000000000000;

    let mut result: bool = true;
    let mut errors: String = "".to_string();

    // parse string
    match parse_message(data) {
        Ok(request) => (),
        Err(error) => return "invalid message",
    }
    let mut request: AuthRequest = parse_message(data).unwrap();

    // check signature
    match validate_signature(&request) {
        Ok(()) => (),
        Err(error) => return "invalid signature",
    }

    // check asset id
    match validate_asset_id(&request, auth_asset_id) {
        Ok(()) => (),
        Err(error) => return "invalid asset",
    }

    // check inputs
    match validate_inputs(&request, auth_gaid_in) {
        Ok(()) => (),
        Err(error) => return "invalid input",
    }

    // check outputs
    match validate_outputs(&request, auth_gaid_out, auth_allow_changes) {
        Ok(()) => (),
        Err(error) => return "invalid output",
    }

    // check amounts
    match validate_amounts(&request, auth_min_amount, auth_max_amount) {
        Ok(()) => (),
        Err(error) => return "invalid amount",
    }

    // send back results
    return "ok"
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![main_api, authorizer_api, authorizer_api_get])
}

fn main() {
    rocket().launch();
}
