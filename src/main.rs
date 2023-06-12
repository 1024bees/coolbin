use coolbin::configuration::get_configuration;

use coolbin::startup::Application;
use coolbin::telemetry::{get_subscriber, init_subscriber};


#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let subscriber = get_subscriber("coolbin".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let app = Application::build(configuration)
        .await
        .expect("App build failed");
    tracing::info!("wasup");
    app.run_until_stopped().await?;
    Ok(())
}
