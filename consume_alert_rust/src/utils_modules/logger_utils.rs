use crate::common::*;

use crate::repository::kafka_repository::*;


/*
    Function responsible for logging
*/
pub fn set_global_logger() {
    let log_directory = "logs"; // Directory to store log files
    let file_prefix = ""; // Prefixes for log files

    // Logger setting
    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default().directory(log_directory).discriminant(file_prefix))
        .rotate(
            Criterion::Age(Age::Day), // daily rotation
            Naming::Timestamps, // Use timestamps for file names
            Cleanup::KeepLogFiles(10) // Maintain up to 10 log files
        )
        .format_for_files(custom_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed: {}", e));
}


/*
    Custom Log Format Function
*/
fn custom_format(w: &mut dyn Write, now: &mut flexi_logger::DeferredNow, record: &Record) -> Result<(), std::io::Error> {
    write!(w, "[{}] [{}] T[{}] {}",
        now.now().format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        std::thread::current().name().unwrap_or("unknown"),
        &record.args())
}

/*

*/
async fn logging_kafka(msg: &str) {
    
    let kafka_producer = get_kafka_producer();
    info!("kafka_producer-1");
    let msg_owned = msg.to_string();

    let handle = task::spawn_blocking(move || {
        kafka_producer.produce_message("consume_alert_rust", &msg_owned).unwrap_or_else(|e| {
            error!("{:?}", e);
        });
    });
    
    info!("kafka_producer-2");

    match handle.await {
        Ok(_) => (),
        Err(e) => error!("Error waiting for task: {:?}", e),
    }
    
}

/*
    error!
*/
pub async fn errork(err: anyhow::Error) {
    
    // file
    error!("{:?}", err);
    logging_kafka(&err.to_string()).await;
}

/*
    info!
*/
pub async fn infok(info: &str) {
    
    // file
    info!("{:?}", info);
    logging_kafka(info).await;
    info!("end");
    // // kafka
    // let kafka_client: Option<&Arc<KafkaRepositoryPub>> = match KAFKA_PRODUCER.get() {
    //     Some(kafka) => Some(kafka),
    //     None => {
    //         error!("[DB Connection Error][infok()] Cannot connect Kafka cluster");
    //         None
    //     }
    // };

    // if let Some(kafka_client) = kafka_client {
    //     kafka_client.logging_kafka(info).await;
    // }
    
}