use apalis::prelude::{Job, JobContext};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Reminder(DateTime<Utc>);

impl From<DateTime<Utc>> for Reminder {
    fn from(value: DateTime<Utc>) -> Self {
        Reminder(value)
    }
}

// set up an identifier for apalis
impl Job for Reminder {
    const NAME: &'static str = "reminder::DailyReminder";
}

pub async fn say_hello_world(job: Reminder, ctx: JobContext) {
    println!("Hello world from `say_hello_world()`!");
    //this lets you use variable stored in the CrontjobData sturct
    let svc = ctx.data_opt::<CronjobData>().unwrap();
    svc.execute(job)
}

#[derive(Clone)]
pub struct CronjobData {
    pub message: String,
}

impl CronjobData {
    fn execute(&self, item: Reminder) {
        println!("{} from CrontjobData::execute()!", &self.message);
    }
}
