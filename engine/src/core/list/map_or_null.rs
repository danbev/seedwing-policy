use crate::core::{Function, FunctionEvaluationResult};
use crate::lang::lir::Bindings;
use crate::runtime::rationale::Rationale;
use crate::runtime::{EvalContext, Output, RuntimeError, World};
use crate::value::RuntimeValue;
use std::future::Future;
use std::pin::Pin;

use crate::lang::PatternMeta;
use std::sync::Arc;

const DOCUMENTATION: &str = include_str!("map-or-null.adoc");

const MAP_FN: &str = "map-fn";

#[derive(Debug)]
pub struct MapOrNull;

impl Function for MapOrNull {
    fn parameters(&self) -> Vec<String> {
        vec![MAP_FN.into()]
    }

    fn metadata(&self) -> PatternMeta {
        PatternMeta {
            documentation: Some(DOCUMENTATION.into()),
            ..Default::default()
        }
    }

    fn call<'v>(
        &'v self,
        input: Arc<RuntimeValue>,
        ctx: &'v EvalContext,
        bindings: &'v Bindings,
        world: &'v World,
    ) -> Pin<Box<dyn Future<Output = Result<FunctionEvaluationResult, RuntimeError>> + 'v>> {
        Box::pin(async move {
            if let Some(map_fn) = bindings.get(MAP_FN) {
                match input.as_ref() {
                    RuntimeValue::List(inputs) => {
                        let mut result = Vec::new();
                        for input in inputs.iter() {
                            if let Some(value) = map_fn
                                .evaluate(input.clone(), ctx, bindings, world)
                                .await?
                                .output()
                            {
                                result.push(value);
                            } else {
                                result.push(RuntimeValue::Null.into());
                            }
                        }
                        Ok(Output::Transform(Arc::new(RuntimeValue::List(result.clone()))).into())
                    }
                    _ => {
                        let msg = "Input is not a list";
                        Ok((Output::None, Rationale::InvalidArgument(msg.into())).into())
                    }
                }
            } else {
                let msg = "Unable to lookup map function";
                Ok((Output::None, Rationale::InvalidArgument(msg.into())).into())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_not_satisfied;
    use crate::runtime::testutil::test_pattern;
    use serde_json::json;

    #[tokio::test]
    async fn test_map_or_null_single_element() {
        let result = test_pattern(
            r#"list::map-or-null<uri::purl>"#,
            RuntimeValue::String(
                "pkg:github/package-url/purl-spec@244fd47e07d1004#everybody/loves/dogs".to_string(),
            ),
        )
        .await;

        assert_not_satisfied!(result);
    }

    #[tokio::test]
    async fn test_map_or_null_list_no_filtering() {
        let result = test_pattern(
            r#"list::map-or-null<uri::purl>"#,
            vec![
                RuntimeValue::String(
                    "pkg:github/package-url/purl-spec@244fd47e07d1004#everybody/loves/dogs"
                        .to_string(),
                ),
                RuntimeValue::String("nomatch".to_string()),
            ],
        )
        .await;

        assert_eq!(
            result.output(),
            Some(Arc::new(
                json!([{
                    "type": "github",
                    "namespace": "package-url",
                    "name": "purl-spec",
                    "version": "244fd47e07d1004",
                    "subpath": "everybody/loves/dogs",
                }, null,
                ])
                .into()
            ))
        );
    }

    #[tokio::test]
    async fn test_map_or_null_list() {
        let result = test_pattern(
            r#"list::map-or-null<uri::purl>"#,
            vec![
                RuntimeValue::String(
                    "pkg:github/package-url/purl-spec@244fd47e07d1004#everybody/loves/dogs"
                        .to_string(),
                ),
                RuntimeValue::String(
                    "pkg:github/package-url/purl-spec@244fd47e07d1004#everybody/loves/cats"
                        .to_string(),
                ),
            ],
        )
        .await;

        assert_eq!(
            result.output(),
            Some(Arc::new(
                json!([{
                    "type": "github",
                    "namespace": "package-url",
                    "name": "purl-spec",
                    "version": "244fd47e07d1004",
                    "subpath": "everybody/loves/dogs",
                }, {
                    "type": "github",
                    "namespace": "package-url",
                    "name": "purl-spec",
                    "version": "244fd47e07d1004",
                    "subpath": "everybody/loves/cats",
                }])
                .into()
            ))
        );
    }
}
