use aide::{
    openapi::{Info, MediaType, OpenApi, Operation, PathItem},
    OperationOutput,
};
use axum::Json;
use pointguard_types::{InvokedTaskPayload, InvokedTaskResponse};
use schemars::JsonSchema;

pub fn new() -> OpenApi {
    let mut api = OpenApi {
        info: Info {
            description: Some("pointguard api".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    register_component::<InvokedTaskPayload>(&mut api, "InvokedTaskPayload");

    let mut operation = Operation::default();
    let _ = aide::transform::TransformOperation::new(&mut operation)
        .input::<Json<InvokedTaskPayload>>()
        .response::<200, Json<InvokedTaskResponse>>();

    api.webhooks.insert(
        "executeTask".to_string(),
        aide::openapi::ReferenceOr::Item(PathItem {
            post: Some(operation),
            ..Default::default()
        }),
    );

    api
}

fn register_component<T: JsonSchema>(api: &mut OpenApi, name: &str) {
    let mut components = api.components.take().unwrap_or_default();
    components.schemas.insert(
        name.to_string(),
        aide::openapi::SchemaObject {
            json_schema: schemars::schema::Schema::Object(schemars::schema_for!(T).schema),
            external_docs: None,
            example: None,
        },
    );
    api.components = Some(components);
}
