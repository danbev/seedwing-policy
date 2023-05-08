use crate::lang::builder::Builder;
use crate::runtime::sources::Ephemeral;
use crate::runtime::RuntimeError;
use crate::runtime::{EvalContext, EvaluationResult};
use crate::wit::engine::Engine;

wit_bindgen::generate!("engine");

struct Exports;

impl Engine for Exports {
    fn version() -> String {
        crate::version().to_string()
    }

    fn eval(
        policies: Vec<String>,
        data: Vec<String>,
        policy: String,
        name: String,
        input: String,
    ) -> String {
        println!("policies: {policies:?}");
        println!("data: {data:?}");
        println!("policy: {policy:?}");
        println!("name: {name:?}");
        println!("input: {input:?}");

        let mut builder = Builder::new();
        // TODO: could we perhaps add a DataSource which is just a hashmap with
        // a name as the key and the value is the policy as a string?
        //builder.data(DirectoryDataSource::new(test_data_dir()));
        let res = builder.build(Ephemeral::new("wit", policy).iter()).unwrap();
        futures::executor::block_on(async {
            let runtime = builder.finish().await;
            let result = runtime
                .unwrap()
                .evaluate(format!("wit::{name}"), input, EvalContext::default())
                .await;
            //Ok(result)
            //Ok::<Result<EvaluationResult, RuntimeError>, E>(result)
            println!("result: {result:?}");
        });

        //let runtime = builder.finish().await.unwrap();
        /*
        let result = runtime
            .evaluate("test::test-pattern", value, EvalContext::default())
            .await;
        */
        "eval result....".to_string()
    }
}

export_engine!(Exports);
