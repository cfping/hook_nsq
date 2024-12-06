# hook_nsq

`hook_nsq` 是一个轻量级的工具，用于在 Windows 系统上快速启动和管理 NSQ 的相关服务，包括 `nsqlookupd`、`nsqd` 和 `nsqadmin`。它支持检测服务的运行状态，并提供简单的配置指南，使您能够快速搭建 NSQ 环境。

---

## 功能特点

1. **快速启动 NSQ 服务：** 
   - 一键启动 `nsqlookupd`、`nsqd` 和 `nsqadmin`。
   - 自动按照配置文件加载服务和参数。
   
2. **状态检测：**
   - 检测指定 NSQ 服务是否正常启动。
   - 提供日志路径以便排查问题。

3. **灵活配置：**
   - 支持通过配置文件指定 NSQ 可执行文件路径及服务参数。
   - 简单易懂的配置方式，适合快速上手。

---

## 使用指南

### 1. 安装与准备

1. 下载并解压 NSQ 的 Windows 版本，确保其包含以下文件：
   - `nsqlookupd.exe`
   - `nsqd.exe`
   - `nsqadmin.exe`

2. 将解压后的路径记录下来，例如：  
   ```
   D:\webx3\nsq-1.3.0.windows-amd64.go1.21.5\bin
   ```

3. 确保 `hook_nsq` 工具放置在任意目录，推荐将其与 NSQ 程序放在同一目录下。

---

### 2. 配置文件

在 `hook_nsq` 的目录下，新建一个 `config.toml` 文件，配置示例如下：

```toml
# NSQ bin 文件的目录
nsq_path = "D:\\webx3\\nsq-1.3.0.windows-amd64.go1.21.5\\bin"

# 服务配置
[[services]]
name = "nsqlookupd"
command = "nsqlookupd.exe"
args = []
log_file = "nsqlookupd.log"

[[services]]
name = "nsqd"
command = "nsqd.exe"
args = ["--lookupd-tcp-address=127.0.0.1:4160"]
log_file = "nsqd.log"

[[services]]
name = "nsqadmin"
command = "nsqadmin.exe"
args = ["--lookupd-http-address=127.0.0.1:4161"]
log_file = "nsqadmin.log"
```

#### 配置说明：
- **`nsq_path`**: 指定 NSQ 可执行文件所在的目录。
- **`[[services]]`**: 定义需要启动的服务，包括：
  - **`name`**: 服务名称，便于识别。
  - **`command`**: 对应服务的可执行文件名称。
  - **`args`**: 启动服务时的命令行参数（可根据需要调整）。
  - **`log_file`**: 指定日志输出文件路径。

---

### 3. 启动工具

1. 确保 `config.toml` 文件已正确配置。
2. 双击运行 `hook_nsq` 或通过命令行启动：
   ```bash
   hook_nsq.exe
   ```

工具会根据配置文件启动指定的服务，并输出相关状态信息到终端。

---

### 4. 检查服务状态

`hook_nsq` 支持自动检测服务状态：
- 如果某服务未正常启动，工具会输出错误提示。
- 日志文件记录在 `config.toml` 中指定的路径下，例如 `nsqlookupd.log`。

---

## 示例场景

1. **快速搭建测试环境：**  
   使用 `hook_nsq` 一键启动所有服务，快速部署 NSQ 消息队列环境。

2. **服务状态管理：**  
   检查某个服务的运行状态，方便排查问题。

3. **配置动态调整：**  
   修改 `config.toml` 文件，调整 NSQ 服务的参数，例如更改监听端口。

---

## 常见问题

1. **工具无法启动服务？**
   - 检查 `nsq_path` 是否正确指向包含 NSQ 可执行文件的目录。
   - 确认可执行文件名与配置中 `command` 一致。

2. **服务日志无输出？**
   - 检查 `log_file` 路径是否可写。
   - 确认服务是否已成功启动。

3. **如何更改服务端口？**
   - 编辑 `config.toml` 中的 `args` 参数，例如：
     ```toml
     args = ["--lookupd-tcp-address=127.0.0.1:5160"]
     ```

---

## 未来计划

- 增加服务自动重启功能。
- 支持自定义检测端口功能。
- 提供更丰富的服务状态监控界面。

---

## 联系我们

如果在使用过程中遇到问题，欢迎通过以下方式联系我们：  
Email: support@hooknsq.com  
GitHub: [hook_nsq](https://github.com/example/hook_nsq)

--- 

Hope this helps! 🎉