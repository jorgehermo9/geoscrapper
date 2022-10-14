use std::error::Error;

mod country;
use country::Country;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let countries = Country::get_all().await?;
    dbg!(countries);
    Ok(())
}
