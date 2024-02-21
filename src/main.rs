use std::str::FromStr;

use apalis::{
    cron::{CronStream, Schedule},
    layers::{DefaultRetryPolicy, Extension},
    postgres::PostgresStorage,
    prelude::{job_fn, timer::TokioTimer, Monitor, WithStorage, WorkerBuilder, WorkerFactory},
};
use crontjobs::{say_hello_world, CronjobData};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::{retry::RetryLayer, ServiceBuilder};

mod crontjobs;

pub struct MyService {
    db: PgPool,
}

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_shared_db::Postgres] conn_string: String,
) -> Result<MyService, shuttle_runtime::Error> {
    let db = PgPoolOptions::new()
        .min_connections(5)
        .max_connections(5)
        .connect(&conn_string)
        .await
        .unwrap();

    Ok(MyService { db })
}

// Customize this struct with things from `shuttle_main` needed in `bind`,
// such as secrets or database connections

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for MyService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let storage = PostgresStorage::new(self.db.clone());
        // set up storage
        storage.setup().await.expect("Unable to run migrations :(");

        let cron_service_ext = CronjobData {
            message: "Hello world".to_string(),
        };

        // create a servicebuilder for the cronjob
        let service = ServiceBuilder::new()
            .layer(RetryLayer::new(DefaultRetryPolicy))
            .layer(Extension(cron_service_ext))
            .service(job_fn(say_hello_world));

        let schedule = Schedule::from_str("* * * * * *").expect("Couldn't start the scheduler!");

        // create a worker that uses the service created from the cronjob
        let worker = WorkerBuilder::new("morning-cereal")
            .with_storage(storage.clone())
            .stream(CronStream::new(schedule).timer(TokioTimer).to_stream())
            .build(service);

        // start your worker up
        Monitor::new()
            .register(worker)
            .run()
            .await
            .expect("Unable to start worker");

        Ok(())
    }
}
