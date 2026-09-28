# 证据

## 复现（修复前，ADB）

- `usb0` DOWN，无 IP
- ConnMan gadget: Powered=False, Tethering=False
- 无 dnsmasq/udhcpd
- `udx710 -H 192.168.66.1 -p 80` 在跑，但地址不在任何网卡上

## 临时救场（不改 loader.sh）

```text
connmanctl enable gadget
connmanctl tether gadget on
ifconfig usb0 192.168.66.1 netmask 255.255.255.0
ip link set dev usb0 up
```

之后：`usb0` UP + 192.168.66.1，Tethering=True，`/api/health` 返回 ok。重启会丢。

## RED / GREEN

- 独立检查 `checks/lan_gateway_plan.rs`：缺 `lan_gateway_plan` 时 rustc 报 E0425（RED）
- 补上计划后 `rustc --test` 通过（GREEN）
- 本机 Windows 无 gcc，`cargo test` 无法编过 sqlite；完整 crate 测试依赖 CI/交叉编译环境

## 根因消除

- `configure_usb_network` 不再执行 `ip route add default via 192.168.66.2`
- `main` 在 bind 前调用 `ensure_lan_gateway()`
