use serde::Deserialize;
use std::{
    ffi::OsStr,
    os::windows::process::CommandExt,
    process::{Child, Command},
    sync::Arc,
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::{
    signal, task,
    time::{sleep, Duration},
};
use tracing::{error, info};
use tracing_subscriber;

#[derive(Debug, Deserialize)]
struct NsqService {
    name: String,
    command: String,
    args: Vec<String>,
    log_file: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    nsq_path: String,
    services: Vec<NsqService>,
}

impl NsqService {
    /// 启动服务
    fn start(&self, nsq_path: &str) -> Option<Child> {
        let full_command = format!("{}/{}", nsq_path, self.command);
        info!("Starting {}...", self.name);
        match Command::new(&full_command)
            .args(&self.args)
            .creation_flags(0x00000008) // CREATE_NO_WINDOW for Windows
            .stdout(std::fs::File::create(&self.log_file).ok()?)
            .stderr(std::fs::File::create(&self.log_file).ok()?)
            .spawn()
        {
            Ok(child) => Some(child),
            Err(err) => {
                error!("Failed to start {}: {}", self.name, err);
                None
            }
        }
    }

    /// 检查服务是否运行
    fn is_running(&self, nsq_path: &str, system: &System) -> bool {
        let full_command = format!("{}/{}", nsq_path, self.command);
        let os_str_command = OsStr::new(&self.name);
        info!("Full Command {} ", &full_command);
        // system.processes_by_name(name)
        // info!("Processes By Name {} ", &system);
        for process in system.processes_by_name(os_str_command) {
            info!("processes_by_name=>{:?}", &process.cmd());
            if process.cmd().iter().any(|arg| {
                info!("Args=>{:?}", arg.to_string_lossy());
                info!("Self Args=>{:?}", self.args.to_owned());
               return  arg.to_string_lossy().to_string().contains(&self.name.to_string());
            }) {
                return true;
            }
        }
        false
    }

    /// 停止服务
    fn stop(&self, nsq_path: &str) {
        let full_command = format!("{}/{}", nsq_path, self.command);
        info!("Stopping {}...", self.name);
        if cfg!(target_os = "windows") {
            let output = Command::new("taskkill")
                .args(&["/IM", &full_command, "/F"])
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    info!("Stopped {} successfully", self.name);
                }
                Ok(output) => {
                    error!(
                        "Failed to stop {}: {}",
                        self.name,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(err) => {
                    error!("Failed to execute taskkill for {}: {}", self.name, err);
                }
            }
        } else {
            error!("Stop command is not implemented for this OS.");
        }
    }
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置文件
    let config: Config = load_config("config.toml").expect("Failed to load configuration");

    let services = Arc::new(config.services);
    let nsq_path = Arc::new(config.nsq_path);

    // 启动服务管理
    let manager_handle = task::spawn(service_manager(services.clone(), nsq_path.clone()));

    // 处理系统信号
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Shutting down gracefully...");
        }
        _ = manager_handle => {
            error!("Service manager task exited unexpectedly!");
        }
    }

    // 停止所有服务
    shutdown_services(services, nsq_path).await;
}

/// 加载配置文件
fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// 服务管理逻辑
async fn service_manager(services: Arc<Vec<NsqService>>, nsq_path: Arc<String>) {
    let mut system = System::new_all();
    loop {
        // 刷新进程信息
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );

        for service in &*services {
            if !service.is_running(&nsq_path, &system) {
                info!("{} is not running. Restarting...", service.name);
                service.start(&nsq_path);
                sleep(Duration::from_secs(5)).await; // 确保进程启动
            } else {
                info!("{} is running.", service.name);
            }
        }

        // 每隔10秒检查一次
        sleep(Duration::from_secs(10)).await;
    }
}

/// 停止所有服务
async fn shutdown_services(services: Arc<Vec<NsqService>>, nsq_path: Arc<String>) {
    for service in services.iter() {
        service.stop(&nsq_path);
    }
}
