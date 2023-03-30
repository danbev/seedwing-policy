use crate::core::{Function, FunctionEvaluationResult};
use crate::lang::{
    lir::{Bindings, InnerPattern},
    PatternMeta, Severity,
};
use crate::runtime::{EvalContext, RuntimeError, World};
use crate::value::RuntimeValue;
use std::cmp::max;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const DOCUMENTATION: &str = include_str!("and.adoc");

const TERMS: &str = "terms";

#[derive(Debug)]
pub struct And;

impl Function for And {
    fn metadata(&self) -> PatternMeta {
        PatternMeta {
            documentation: DOCUMENTATION.into(),
            ..Default::default()
        }
    }

    fn parameters(&self) -> Vec<String> {
        vec![TERMS.into()]
    }

    fn call<'v>(
        &'v self,
        input: Arc<RuntimeValue>,
        ctx: &'v EvalContext,
        bindings: &'v Bindings,
        world: &'v World,
    ) -> Pin<Box<dyn Future<Output = Result<FunctionEvaluationResult, RuntimeError>> + 'v>> {
        Box::pin(async move {
            if let Some(terms) = bindings.get(TERMS) {
                if let InnerPattern::List(terms) = terms.inner() {
                    let mut severity = Severity::None;
                    let mut supporting = Vec::new();
                    let mut terms = terms.clone();
                    terms.sort_by_key(|a| a.order(world));

                    for term in terms {
                        let result = term.evaluate(input.clone(), ctx, bindings, world).await?;
                        severity = max(severity, result.severity());
                        supporting.push(result)
                    }

                    return Ok((severity, supporting).into());
                }
            }

            Ok(Severity::Error.into())
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lang::builder::Builder;
    use crate::runtime::sources::Ephemeral;
    use crate::{assert_not_satisfied, assert_satisfied};
    use serde_json::json;

    #[tokio::test]
    async fn call_matching_both_arms() {
        let src = Ephemeral::new(
            "test",
            r#"
            pattern left = {
              first_name: "bob",
            }

            pattern right = {
              last_name: "mcw",
            }
            pattern test-and = left && right
        "#,
        );

        let mut builder = Builder::new();

        let _result = builder.build(src.iter());
        let runtime = builder.finish().await.unwrap();

        let result = runtime
            .evaluate(
                "test::test-and",
                json!(
                    {
                        "first_name": "bob",
                        "last_name": "mcw"
                    }
                ),
                EvalContext::default(),
            )
            .await
            .unwrap();

        assert_satisfied!(result);
    }

    #[tokio::test]
    async fn call_matching_only_left_arm() {
        let src = Ephemeral::new(
            "test",
            r#"
            pattern left = {
              first_name: "bob",
            }

            pattern right = {
              last_name: "mcw",
            }
            pattern test-and = left && right
        "#,
        );

        let mut builder = Builder::new();

        let _result = builder.build(src.iter());

        let runtime = builder.finish().await.unwrap();

        let result = runtime
            .evaluate(
                "test::test-and",
                json!(
                    {
                        "first_name": "bob"
                    }
                ),
                EvalContext::default(),
            )
            .await
            .unwrap();

        assert_not_satisfied!(result);
    }

    #[tokio::test]
    async fn call_matching_only_right_arm() {
        let src = Ephemeral::new(
            "test",
            r#"
            pattern left = {
              first_name: "bob",
            }

            pattern right = {
              last_name: "mcw",
            }
            pattern test-and = left && right
        "#,
        );

        let mut builder = Builder::new();

        let _result = builder.build(src.iter());

        let runtime = builder.finish().await.unwrap();

        let result = runtime
            .evaluate(
                "test::test-and",
                json!(
                    {
                        "last_name": "mcw"
                    }
                ),
                EvalContext::default(),
            )
            .await
            .unwrap();

        assert_not_satisfied!(result);
    }

    #[tokio::test]
    async fn call_matching_no_arms() {
        let src = Ephemeral::new(
            "test",
            r#"
            pattern left = {
              first_name: "bob",
            }

            pattern right = {
              last_name: "mcw",
            }
            pattern test-and = left && right
        "#,
        );

        let mut builder = Builder::new();

        let _result = builder.build(src.iter());

        let runtime = builder.finish().await.unwrap();

        let result = runtime
            .evaluate(
                "test::test-and",
                json!(
                    {
                        "first_name": "jim",
                        "last_name": "crossley"
                    }
                ),
                EvalContext::default(),
            )
            .await
            .unwrap();

        assert_not_satisfied!(result);
    }
}
