use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::env::current_dir;
use std::fs::create_dir_all;
use multi_token_agreement::msg::{AgreementResponse, AgreementsResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};
use multi_token_agreement::state::Agreement;

fn main() {
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
