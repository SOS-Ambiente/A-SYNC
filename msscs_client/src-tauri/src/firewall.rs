// Firewall management module for Windows
use std::process::Command;

/// Add Windows Firewall rule for the application
pub fn add_firewall_rule(app_name: &str, app_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Check if rule already exists
        let check_output = Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "show",
                "rule",
                &format!("name={}", app_name),
            ])
            .output()
            .map_err(|e| format!("Failed to check firewall rule: {}", e))?;

        if check_output.status.success() {
            let output_str = String::from_utf8_lossy(&check_output.stdout);
            if output_str.contains("No rules match") {
                // Rule doesn't exist, create it
                tracing::info!("🛡️  Creating Windows Firewall rule for {}", app_name);
                
                // Add inbound rule for TCP
                let tcp_result = Command::new("netsh")
                    .args(&[
                        "advfirewall",
                        "firewall",
                        "add",
                        "rule",
                        &format!("name={} TCP", app_name),
                        "dir=in",
                        "action=allow",
                        &format!("program={}", app_path),
                        "enable=yes",
                        "protocol=TCP",
                    ])
                    .output()
                    .map_err(|e| format!("Failed to add TCP firewall rule: {}", e))?;

                if !tcp_result.status.success() {
                    return Err(format!(
                        "Failed to add TCP firewall rule: {}",
                        String::from_utf8_lossy(&tcp_result.stderr)
                    ));
                }

                // Add inbound rule for UDP (for QUIC)
                let udp_result = Command::new("netsh")
                    .args(&[
                        "advfirewall",
                        "firewall",
                        "add",
                        "rule",
                        &format!("name={} UDP", app_name),
                        "dir=in",
                        "action=allow",
                        &format!("program={}", app_path),
                        "enable=yes",
                        "protocol=UDP",
                    ])
                    .output()
                    .map_err(|e| format!("Failed to add UDP firewall rule: {}", e))?;

                if !udp_result.status.success() {
                    return Err(format!(
                        "Failed to add UDP firewall rule: {}",
                        String::from_utf8_lossy(&udp_result.stderr)
                    ));
                }

                tracing::info!("✅ Firewall rules created successfully");
                Ok(())
            } else {
                tracing::info!("✅ Firewall rule already exists");
                Ok(())
            }
        } else {
            Err("Failed to check existing firewall rules".to_string())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // No firewall configuration needed on non-Windows platforms
        Ok(())
    }
}

/// Remove Windows Firewall rule for the application
pub fn remove_firewall_rule(app_name: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        tracing::info!("🛡️  Removing Windows Firewall rules for {}", app_name);
        
        // Remove TCP rule
        let _ = Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                &format!("name={} TCP", app_name),
            ])
            .output();

        // Remove UDP rule
        let _ = Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                &format!("name={} UDP", app_name),
            ])
            .output();

        tracing::info!("✅ Firewall rules removed");
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

/// Check if the application has firewall access
pub fn check_firewall_access(app_name: &str) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "show",
                "rule",
                &format!("name={}", app_name),
            ])
            .output()
            .map_err(|e| format!("Failed to check firewall: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(!output_str.contains("No rules match"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(true)
    }
}

/// Request admin privileges to add firewall rule
pub fn request_firewall_access_with_elevation(app_name: &str, app_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        tracing::info!("🔐 Requesting elevated privileges for firewall access...");

        // Create a PowerShell script to add firewall rules
        let script = format!(
            r#"
            Start-Process netsh -ArgumentList 'advfirewall firewall add rule name="{} TCP" dir=in action=allow program="{}" enable=yes protocol=TCP' -Verb RunAs -Wait
            Start-Process netsh -ArgumentList 'advfirewall firewall add rule name="{} UDP" dir=in action=allow program="{}" enable=yes protocol=UDP' -Verb RunAs -Wait
            "#,
            app_name, app_path, app_name, app_path
        );

        let result = Command::new("powershell")
            .args(&["-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to request elevation: {}", e))?;

        if result.status.success() {
            tracing::info!("✅ Firewall access granted");
            Ok(())
        } else {
            Err(format!(
                "Failed to add firewall rules: {}",
                String::from_utf8_lossy(&result.stderr)
            ))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}
