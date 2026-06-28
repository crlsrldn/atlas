#!/bin/bash
set -e

# I will use a simple python script to safely replace the logic inside resolve_real_debrid_download
python3 -c '
import sys

with open("core/src/api/resolve.rs", "r") as f:
    content = f.read()

old_block = """    let info = client
        .get(info_url)
        .bearer_auth(api_key)
        .send()
        .await
        .ok()?
        .json::<RDInfoResponse>()
        .await
        .ok()?;

    let link = info.links.first()?;"""

new_block = """    // Retry loop to wait for Real-Debrid backend to transition torrent to downloaded state
    let mut link = None;
    for _ in 0..10 {
        if let Ok(res) = client.get(&info_url).bearer_auth(api_key).send().await {
            if let Ok(info) = res.json::<RDInfoResponse>().await {
                if let Some(l) = info.links.first() {
                    link = Some(l.clone());
                    break;
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    let link = link?;"""

if old_block in content:
    content = content.replace(old_block, new_block)
    with open("core/src/api/resolve.rs", "w") as f:
        f.write(content)
    print("Replaced successfully!")
else:
    print("Old block not found!")
    sys.exit(1)
'
