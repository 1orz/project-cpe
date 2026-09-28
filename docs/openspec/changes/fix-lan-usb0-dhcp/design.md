# 设计：启动时拉起局域网网关

## 方案

从 `configure_usb_network()` 抽出「只修局域网、不改默认路由」的函数（例如 `ensure_lan_gateway`）：

1. `connmanctl enable gadget`
2. `connmanctl tether gadget on`（由 ConnMan 开 DHCP）
3. 给 `usb0` 设 `192.168.66.1/24` 并 `up`
4. 可选：`sipa_usb0 down`（与现网脚本一致）
5. **禁止** `ip route add default via 192.168.66.2`

在 `main` 绑定监听端口之前调用。这样即使 `loader.sh` 仍是 `-H 192.168.66.1`，地址也已存在。

热切换 USB 模式继续走原 `configure_usb_network()`，但去掉其中的默认路由劫持，或改为调用上述函数。

## 不做什么

- 不改设备上的 `loader.sh` 作为 OTA 交付物（`pack-ota.sh` 不打包它）。
- 不把 `route_test.sh` 拷进仓库当唯一修复。
- 不新增 capability / API。

## 验证

- 单元测试：网关配置命令列表不含默认路由 via `192.168.66.2`。
- 真机：重启或新 OTA 后，以太网 2 能拿到 `192.168.66.x` 并打开后台。
