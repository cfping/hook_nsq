use std::{ffi::OsStr, os::windows::process::CommandExt, process::{Child, Command}, sync::Arc};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::{
    signal,
    task,
    time::{sleep, Duration},
};
use tracing::{error, info};
use tracing_subscriber;

#[derive(Debug)]
struct NsqService {
    name: &'static str,
    command: &'static str,
    args: Vec<&'static str>,
    log_file: &'static str,
}

impl NsqService {
    // 启动服务
    fn start(&self) -> Option<Child> {
        info!("Starting {}...", self.name);
        match Command::new(self.command)
            .args(&self.args)
            .creation_flags(0x00000008) // CREATE_NO_WINDOW for Windows
            .stdout(std::fs::File::create(self.log_file).unwrap())
            .stderr(std::fs::File::create(self.log_file).unwrap())
            .spawn()
        {
            Ok(child) => Some(child),
            Err(err) => {
                error!("Failed to start {}: {}", self.name, err);
                None
            }
        }
    }

    // 检查服务是否运行
    fn is_running(&self, system: &System) -> bool {
        let os_str_command = OsStr::new(self.command); // 转换为 OsStr 类型
        info!("Checking if {} is running...", self.name);
        // system.processes_to_update();
        for process in system.processes_by_name(os_str_command) {
            info!("process=>{:?}", &process.cmd());
            if process.cmd().iter().any(|arg| {
                info!("process arg=>{:?}", &arg);
                self.args.contains(&arg.to_str().unwrap())
            }) {
                return true;
            }
        }
        false
    }

    // 停止服务
    fn stop(&self) {
        info!("Stopping {}...", self.name);
        if cfg!(target_os = "windows") {
            // 使用 taskkill 命令
            let output = Command::new("taskkill")
                .args(&["/IM", self.command, "/F"])
                .output()
                .expect("Failed to execute taskkill");
            if output.status.success() {
                info!("Stopped {} successfully", self.name);
            } else {
                error!(
                    "Failed to stop {}: {}",
                    self.name,
                    String::from_utf8_lossy(&output.stderr)
                );
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

    // 定义服务列表
    let services = Arc::new(vec![
        NsqService {
            name: "nsqlookupd",
            command: "nsqlookupd.exe",
            args: vec![],
            log_file: "nsqlookupd.log",
        },
        NsqService {
            name: "nsqd",
            command: "nsqd.exe",
            args: vec!["--lookupd-tcp-address=127.0.0.1:4160"],
            log_file: "nsqd.log",
        },
        NsqService {
            name: "nsqadmin",
            command: "nsqadmin.exe",
            args: vec!["--lookupd-http-address=127.0.0.1:4161"],
            log_file: "nsqadmin.log",
        },
    ]);

    // 启动服务管理
    let services_clone = services.clone();
    let manager_handle = task::spawn(async move {
        let mut system = System::new_all();
        loop {
            // 刷新所有进程信息
            system.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                     true,
                     ProcessRefreshKind::everything(),
                 );

            for service in &*services_clone {
                // 启动服务并稍作等待
                if !service.is_running(&system) {
                    info!("{} is not running. Restarting...", service.name);
                    service.start();
                    // 等待一段时间以确保进程启动
                    sleep(Duration::from_secs(5)).await;
                } else {
                    info!("{} is running.", service.name);
                }
            }

            // 每隔10秒检查一次
            sleep(Duration::from_secs(10)).await;
        }
    });

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
    shutdown_services(services.clone()).await;
}

// 停止所有服务的辅助函数
async fn shutdown_services(services: Arc<Vec<NsqService>>) {
    for ele in services.iter() {
        ele.stop();
    }
}
