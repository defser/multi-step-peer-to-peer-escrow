use std::env;
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::env::current_dir;
use std::fs::create_dir_all;
use peer_to_peer_token_swap::msg::{AgreementResponse, AgreementsResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};
use peer_to_peer_token_swap::state::Agreement;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(AgreementResponse), &out_dir);
    export_schema(&schema_for!(AgreementsResponse), &out_dir);
    export_schema(&schema_for!(TokenInfo), &out_dir);
    export_schema(&schema_for!(Agreement), &out_dir);
}
