from pathlib import Path

path = Path(r"d:\project-cpe\backend\src\main.rs")
text = path.read_text(encoding="utf-8")

needle = """    info!(\"Copyright (c) 2025 1orz - https://github.com/1orz/project-cpe\");

    // 绑定端口，如果被占用则轮询等待（最多 30 秒）
    let listener = bind_with_retry(&bind_addr, 30).await?;
"""

replacement = """    info!(\"Copyright (c) 2025 1orz - https://github.com/1orz/project-cpe\");

    // Bring up usb0 + ConnMan gadget DHCP before bind so -H 192.168.66.1 works.
    if let Err(error) = usb_switch::ensure_lan_gateway() {
        warn!(error = %error, \"Failed to ensure LAN gateway on usb0\");
    }

    // 绑定端口，如果被占用则轮询等待（最多 30 秒）
    let listener = bind_with_retry(&bind_addr, 30).await?;
"""

if needle not in text:
    raise SystemExit("main.rs bind site not found")
path.write_text(text.replace(needle, replacement, 1), encoding="utf-8", newline="\n")
print("main.rs updated ok")
