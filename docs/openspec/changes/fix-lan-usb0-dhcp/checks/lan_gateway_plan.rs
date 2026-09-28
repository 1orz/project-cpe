#[derive(Debug, PartialEq)]
pub struct LanGatewayStep {
    pub program: String,
    pub args: Vec<String>,
}

fn step(program: &str, args: &[&str]) -> LanGatewayStep {
    LanGatewayStep {
        program: program.to_string(),
        args: args.iter().map(|s| (*s).to_string()).collect(),
    }
}

const USB_INTERFACE_IP: &str = "192.168.66.1";

pub fn lan_gateway_plan() -> Vec<LanGatewayStep> {
    vec![
        step("connmanctl", &["enable", "gadget"]),
        step("connmanctl", &["tether", "gadget", "on"]),
        step(
            "ifconfig",
            &["usb0", USB_INTERFACE_IP, "netmask", "255.255.255.0"],
        ),
        step("ip", &["link", "set", "dev", "usb0", "up"]),
        step("ifconfig", &["sipa_usb0", "down"]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lan_gateway_plan_enables_tether_without_hijacking_default_route() {
        let plan = lan_gateway_plan();
        assert!(plan.iter().any(|s| {
            s.program == "connmanctl"
                && s.args
                    == [
                        "tether".to_string(),
                        "gadget".to_string(),
                        "on".to_string(),
                    ]
        }));
        assert!(plan.iter().any(|s| s.args.iter().any(|a| a == USB_INTERFACE_IP)));
        assert!(!plan.iter().any(|s| {
            s.program == "ip"
                && s.args.windows(4).any(|w| {
                    w == [
                        "route".to_string(),
                        "add".to_string(),
                        "default".to_string(),
                        "via".to_string(),
                    ]
                })
        }));
    }
}
