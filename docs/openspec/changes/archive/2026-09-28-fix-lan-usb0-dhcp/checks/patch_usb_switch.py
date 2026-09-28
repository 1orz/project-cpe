from pathlib import Path

path = Path(r"d:\project-cpe\backend\src\usb_switch.rs")
text = path.read_text(encoding="utf-8")

marker = "/// 配置 USB 网络接口"
start = text.find(marker)
if start < 0:
    raise SystemExit("configure marker not found")

end = text.find("/// USB 模式读取结果", start)
if end < 0:
    raise SystemExit("end marker not found")

new_block = r'''/// One shell step used to bring up the LAN gateway on usb0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanGatewayStep {
    pub program: String,
    pub args: Vec<String>,
}

fn lan_gateway_step(program: &str, args: &[&str]) -> LanGatewayStep {
    LanGatewayStep {
        program: program.to_string(),
        args: args.iter().map(|s| (*s).to_string()).collect(),
    }
}

/// Ordered commands that enable ConnMan gadget DHCP and assign 192.168.66.1.
///
/// Must not hijack the cellular default route.
pub fn lan_gateway_plan() -> Vec<LanGatewayStep> {
    vec![
        lan_gateway_step("connmanctl", &["enable", "gadget"]),
        lan_gateway_step("connmanctl", &["tether", "gadget", "on"]),
        lan_gateway_step(
            "ifconfig",
            &["usb0", USB_INTERFACE_IP, "netmask", "255.255.255.0"],
        ),
        lan_gateway_step("ip", &["link", "set", "dev", "usb0", "up"]),
        lan_gateway_step("ifconfig", &["sipa_usb0", "down"]),
    ]
}

/// Bring up usb0 LAN gateway + DHCP without changing the default route.
pub fn ensure_lan_gateway() -> Result<(), String> {
    for step in lan_gateway_plan() {
        let _ = Command::new(&step.program).args(&step.args).output();
    }
    Ok(())
}

/// 配置 USB 网络接口
///
/// 热切换后拉起局域网网关；不修改蜂窝默认路由。
fn configure_usb_network() -> Result<(), String> {
    // 等待接口出现
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 先关闭再重新启用（避免 Already enabled），再走统一拉起路径
    let _ = Command::new("connmanctl")
        .args(["tether", "gadget", "off"])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = Command::new("connmanctl")
        .args(["disable", "gadget"])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(200));

    ensure_lan_gateway()?;

    // 设置 MAC 地址（与 gadget 约定一致）
    let _ = Command::new("ifconfig")
        .args(["usb0", "hw", "ether", USB_INTERFACE_MAC])
        .output();

    // 启用 SFP 硬件转发加速
    let _ = enable_sfp_acceleration();

    // 标记配置完成
    let _ = fs::write("/tmp/sipa_usb0_ok", "");

    // 输出当前网络配置到内核日志（用于调试）
    let _ = Command::new("sh")
        .args(["-c", "ifconfig > /dev/kmsg 2>/dev/null"])
        .output();

    Ok(())
}

'''

text = text[:start] + new_block + text[end:]

old_test = '''#[cfg(test)]
mod tests {
    use super::{lan_gateway_plan, USB_INTERFACE_IP};

    #[test]
    fn lan_gateway_plan_enables_tether_without_hijacking_default_route() {
        let plan = lan_gateway_plan();
        assert!(
            plan.iter().any(|step| {
                step.program == "connmanctl" && step.args == ["tether", "gadget", "on"]
            }),
            "plan must enable connman gadget DHCP"
        );
        assert!(
            plan.iter().any(|step| step.args.contains(&USB_INTERFACE_IP)),
            "plan must assign the LAN gateway address"
        );
        assert!(
            !plan.iter().any(|step| {
                step.program == "ip"
                    && step.args.windows(4).any(|window| {
                        window == ["route", "add", "default", "via"]
                    })
            }),
            "plan must not steal the default route from cellular"
        );
    }
}
'''

new_test = '''#[cfg(test)]
mod tests {
    use super::{lan_gateway_plan, USB_INTERFACE_IP};

    #[test]
    fn lan_gateway_plan_enables_tether_without_hijacking_default_route() {
        let plan = lan_gateway_plan();
        assert!(
            plan.iter().any(|step| {
                step.program == "connmanctl"
                    && step.args
                        == [
                            "tether".to_string(),
                            "gadget".to_string(),
                            "on".to_string(),
                        ]
            }),
            "plan must enable connman gadget DHCP"
        );
        assert!(
            plan.iter().any(|step| step.args.iter().any(|arg| arg == USB_INTERFACE_IP)),
            "plan must assign the LAN gateway address"
        );
        assert!(
            !plan.iter().any(|step| {
                step.program == "ip"
                    && step.args.windows(4).any(|window| {
                        window
                            == [
                                "route".to_string(),
                                "add".to_string(),
                                "default".to_string(),
                                "via".to_string(),
                            ]
                    })
            }),
            "plan must not steal the default route from cellular"
        );
    }
}
'''

if old_test not in text:
    raise SystemExit("test block not found")
text = text.replace(old_test, new_test, 1)

path.write_text(text, encoding="utf-8", newline="\n")
print("usb_switch.rs updated ok")
