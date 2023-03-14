use crate::core::{Function, FunctionEvaluationResult};
use crate::lang::lir::{Bindings, InnerPattern};
use crate::lang::ValuePattern;
use crate::runtime::rationale::Rationale;

use crate::lang::PatternMeta;
use crate::runtime::{EvalContext, World};
use crate::runtime::{Output, RuntimeError};
use crate::value::RuntimeValue;
use sigstore::tuf::SigstoreRepository;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::spawn_blocking;

#[derive(Debug)]
pub struct VerifyBlob;

const VERIFY_BLOB_DOCUMENATION: &str = include_str!("verify-blob.adoc");
const CERTIFICATE: &str = "certificate";
const SIGNATURE: &str = "signature";
const CHECKOUT_DIR: &str = "checkout_dir";

impl Function for VerifyBlob {
    fn order(&self) -> u8 {
        200
    }

    fn parameters(&self) -> Vec<String> {
        vec![CERTIFICATE.into(), SIGNATURE.into(), CHECKOUT_DIR.into()]
    }

    fn metadata(&self) -> PatternMeta {
        PatternMeta {
            documentation: Some(VERIFY_BLOB_DOCUMENATION.into()),
            ..Default::default()
        }
    }

    fn call<'v>(
        &'v self,
        input: Arc<RuntimeValue>,
        _ctx: &'v EvalContext,
        bindings: &'v Bindings,
        _world: &'v World,
    ) -> Pin<Box<dyn Future<Output = Result<FunctionEvaluationResult, RuntimeError>> + 'v>> {
        Box::pin(async move {
            if let Option::Some(blob) = input.try_get_string() {
                let cert = match get_parameter(CERTIFICATE, bindings) {
                    Ok(value) => value,
                    Err(msg) => {
                        return invalid_arg(msg);
                    }
                };
                let sig = match get_parameter(SIGNATURE, bindings) {
                    Ok(value) => value,
                    Err(msg) => {
                        return invalid_arg(msg);
                    }
                };
                let checkout_dir = match get_parameter(CHECKOUT_DIR, bindings) {
                    Ok(value) => Some(value),
                    Err(_) => None,
                };

                log::debug!("certificates: {}", cert);
                log::debug!("signature: {}", cert);
                log::debug!("checkout_dir: {:?}", &checkout_dir);
                log::debug!("blob: {}", blob);

                // Fetch from The Update Framework (TUF) repository
                let _repo: sigstore::errors::Result<SigstoreRepository> =
                    spawn_blocking(move || {
                        if checkout_dir.is_some() {
                            sigstore::tuf::SigstoreRepository::fetch(Some(Path::new(
                                &checkout_dir.unwrap(),
                            )))
                        } else {
                            sigstore::tuf::SigstoreRepository::fetch(None)
                        }
                    })
                    .await
                    .unwrap();

                match sigstore::cosign::verify_blob(&cert, &sig, &blob.as_bytes()) {
                    Ok(_) => {
                        return Ok(Output::Transform(Arc::new(RuntimeValue::Boolean(true))).into())
                    }
                    Err(e) => {
                        log::error!("verify_blob failed with {:?}", e);
                        return Ok(Output::Transform(Arc::new(RuntimeValue::Boolean(false))).into());
                    }
                }
            }
            Ok(Output::None.into())
        })
    }
}

fn get_parameter(param: &str, bindings: &Bindings) -> Result<String, String> {
    match bindings.get(param) {
        Some(pattern) => match pattern.inner() {
            InnerPattern::Const(pattern) => match pattern {
                ValuePattern::String(value) => Ok(value.to_string()),
                _ => Err(format!("invalid type specified for {param} parameter")),
            },
            _ => Err(format!("invalid type specified for {param} parameter")),
        },
        None => Err(format!("invalid type specified for {param} parameter")),
    }
}

fn invalid_arg(msg: impl Into<String>) -> Result<FunctionEvaluationResult, RuntimeError> {
    Ok((Output::None, Rationale::InvalidArgument(msg.into())).into())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::lang::builder::Builder;
    use crate::runtime::sources::Ephemeral;

    #[actix_rt::test]
    async fn verify_blob() {
        let _ = env_logger::builder().is_test(true).try_init();

        let src = Ephemeral::new(
            "test",
            r#"
            pattern certificate = "LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUNwekNDQWk2Z0F3SUJBZ0lVVmtLeDdsbVV6MG5acldTUnZMZkQxc24vdFhzd0NnWUlLb1pJemowRUF3TXcKTnpFVk1CTUdBMVVFQ2hNTWMybG5jM1J2Y21VdVpHVjJNUjR3SEFZRFZRUURFeFZ6YVdkemRHOXlaUzFwYm5SbApjbTFsWkdsaGRHVXdIaGNOTWpNd016RXpNVEUwTVRFMFdoY05Nak13TXpFek1URTFNVEUwV2pBQU1Ga3dFd1lICktvWkl6ajBDQVFZSUtvWkl6ajBEQVFjRFFnQUVJZGdPVkdYQk1Jbk50M0JRRkF1a2Y5alpIa3BzYTJHd0p4d0wKQzVXbFA4SDZDVTFMU2Rtc1p5Zk9aZXBHSUROb1hhUDF2Z2RLckdLRUM1NVdYVUlid0tPQ0FVMHdnZ0ZKTUE0RwpBMVVkRHdFQi93UUVBd0lIZ0RBVEJnTlZIU1VFRERBS0JnZ3JCZ0VGQlFjREF6QWRCZ05WSFE0RUZnUVVVUHppCnBJbHIxYlhPOUs2NFVHQlJVWDFlOEpBd0h3WURWUjBqQkJnd0ZvQVUzOVBwejFZa0VaYjVxTmpwS0ZXaXhpNFkKWkQ4d0p3WURWUjBSQVFIL0JCMHdHNEVaWkdGdWFXVnNMbUpsZG1WdWFYVnpRR2R0WVdsc0xtTnZiVEFzQmdvcgpCZ0VFQVlPL01BRUJCQjVvZEhSd2N6b3ZMMmRwZEdoMVlpNWpiMjB2Ykc5bmFXNHZiMkYxZEdnd2dZb0dDaXNHCkFRUUIxbmtDQkFJRWZBUjZBSGdBZGdEZFBUQnF4c2NSTW1NWkhoeVpaemNDb2twZXVONDhyZitIaW5LQUx5bnUKamdBQUFZYmF4azUvQUFBRUF3QkhNRVVDSVFEbDI2ejdBV3ljb1pJUWwzSVlERjlBYTBoSVMwMW1oY3JtM3YrVgo5TzJYaXdJZ2VlbUt0UUZWZHBXVHM4dVAzMlY2NzIxbkNMVjVySGxnbnE1K2loc1pRL1V3Q2dZSUtvWkl6ajBFCkF3TURad0F3WkFJd0xoV2h5ai84aW9SNlNEQXB6SEFub3FkUnpJaEprcmkweHZWTjIyV09uSG1ydjFEQis2QWkKcEprRGs1L1FFcEhZQWpCcHIzWWNPYndqYXFLRlZtc1lKa0N0MnZqQ0lYUm0zTCtzRSt6UW9MaklKU09ndGRnUQpDZHVvMUsyMndzUHBzdVk9Ci0tLS0tRU5EIENFUlRJRklDQVRFLS0tLS0K"
            pattern signature = "MEUCIQDCWmgVo1nHK4wh/XWK59LlRVfSstxNA7iMAriNdr235gIgZvPxXb1SVpdNNVwdROtj16prTLKI6vlzmHhw15WHMms="

            // TODO: (danbev) Add support for resolving $HOME, and '~'.
            pattern checkout_dir = "$HOME/.sigstore/root/targets"

            pattern verify_blob = sigstore::verify-blob<certificate, signature, checkout_dir>
        "#,
        );

        let mut builder = Builder::new();
        let _result = builder.build(src.iter());
        let runtime = builder.finish().await.unwrap();
        let blob = "something\n";
        let result = runtime
            .evaluate("test::verify_blob", blob, EvalContext::default())
            .await;
        assert!(result.as_ref().unwrap().satisfied());

        let output = result.as_ref().unwrap().output().unwrap();
        assert_eq!(output.is_boolean(), true);
        assert_eq!(output.try_get_boolean().unwrap(), true);
    }
}