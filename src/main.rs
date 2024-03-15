mod opts;

#[tokio::main]
async fn main() {
    let config = opts::Args::read_config();
    println!("Config: {:?}", config);
}
