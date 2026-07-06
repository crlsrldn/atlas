use redis::AsyncCommands;
use std::env;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let client = redis::Client::open("rediss://default:gQAAAAAAAmb7AAIgcDFiMjgzMDNmOTg1OWM0Mjg2ODUxZWY5NDAxNzU5ZTA0Ng@shining-ghost-157435.upstash.io:6379")?;
    let mut con = client.get_connection_manager().await?;
    
    redis::cmd("SETEX").arg("test_key").arg(60).arg("hello").query_async::<_, ()>(&mut con).await?;
    println!("Write successful!");
    
    let res: String = redis::cmd("GET").arg("test_key").query_async(&mut con).await?;
    println!("Read: {}", res);
    
    Ok(())
}
