mod discord;
mod docker;
mod servers;

use docker::ServerEventMessage;
use dotenv::dotenv;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let servers = servers::get_servers().await;
    let (tx, mut rx) = broadcast::channel::<ServerEventMessage>(8);

    match servers {
        Ok(servers) => {
            for s in servers {
                tokio::spawn(docker::handle_server(s, tx.clone()));
            }
            tokio::spawn(discord::handle_connection(tx.clone()));
        }
        Err(e) => {
            println!("Error getting server list: {}", e);
            return;
        }
    }

    // Loop keeps the app alive - there needs to be more logic in here
    loop {
        let _msg = rx.recv().await.unwrap();
    }
}
