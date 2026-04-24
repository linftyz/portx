# portx

面向 macOS 和 Linux 的现代端口与进程管理工具。

`portx` 是一个以终端为中心的工具，用来查看监听端口、反查所属进程、识别暴露范围，并在需要时快速处理进程。它的目标不是单纯替代某一个命令，而是把 `netstat`、`lsof`、`ps` 这些常见工作流整合成更顺手的一套体验。

[English README](./README.md)

## 当前状态

`portx` 当前聚焦于 macOS 和 Linux。

目前已经实现：

- `list`：查看监听端口
- `info`：查看指定端口详情
- `find`：按进程名查找监听端口
- `kill`：结束某个端口背后的进程
- `watch`：每秒刷新一次端口状态
- `tui`：交互式终端界面
- `list`、`info`、`find` 的 `--json` 输出

后续计划：

- Windows 支持
- 继续打磨 TUI 的交互和布局
- 基于 JSON 输出做更好的自动化集成

## 为什么做 portx

- 自动识别监听范围：`PUBLIC`、`LAN`、`LOCAL`
- 以“端口”为中心，而不是直接把 socket 原始信息甩给你
- 自动关联进程信息，包括 PID、名称、命令、用户、资源占用
- 同时提供适合人读的文本输出和适合工具处理的稳定 JSON 输出
- 提供交互式 TUI，方便持续观察

## 暴露范围判定

`portx` 会自动判断一个监听地址属于哪一类：

- `PUBLIC`：`0.0.0.0`、`::` 以及其他公网可达地址
- `LAN`：`10.0.0.0/8`、`172.16.0.0/12`、`192.168.0.0/16`、`fc00::/7`、`fe80::/10`
- `LOCAL`：`127.0.0.0/8`、`::1`

如果监听绑定在公网地址或通配地址上，`portx` 会给出 warning，帮助你更快发现高风险暴露。

## 安装

### 从源码构建

```bash
git clone <your-repo-url>
cd portx
cargo build
```

### 直接运行

```bash
cargo run -- list
```

### 安装到本地

```bash
cargo install --path .
```

## 用法

```bash
portx
portx list [--scope public|lan|local] [--json]
portx <port>
portx info <port> [--pid <pid>] [--json]
portx find <process_name> [--scope public|lan|local] [--json]
portx kill <port> [--pid <pid>] [--force] [--yes]
portx watch <port> [--pid <pid>]
portx tui
```

### 说明

- `portx` 不带子命令时，等价于 `portx list`。
- `portx 3000` 会自动规范化为 `portx info 3000`。
- `kill` 默认是保守策略：
  - 默认发送 `SIGTERM`，只有加 `--force` 才会强制结束
  - 在交互式终端里会先进行确认
  - 非交互环境下必须显式传 `--yes`
  - 如果同一个端口对应多个 PID，必须显式传 `--pid`
- 某些进程字段可能因为系统权限不可见。此时文本输出会显示 `N/A`，JSON 输出会使用 `null`。

## 示例

### 查看全部监听端口

```bash
portx
```

### 只看本地监听

```bash
portx list --scope local
```

### 查看 5432 端口详情

```bash
portx info 5432
```

### 在共享端口上限定某个 PID

```bash
portx info 3000 --pid 4242
```

### 查找所有 Node.js 监听

```bash
portx find node
```

### 在脚本中使用 JSON

```bash
portx list --json
portx info 5432 --json
portx find postgres --json
```

### 优雅终止某个端口背后的进程

```bash
portx kill 3000
```

### 强制结束并跳过确认

```bash
portx kill 3000 --pid 4242 --force --yes
```

### 实时观察某个端口

```bash
portx watch 5432
```

### 打开交互式 TUI

```bash
portx tui
```

## TUI 快捷键

- `Up` / `Down`：移动选择
- `Enter`：切换详情聚焦模式
- `k`：打开 kill 确认框
- `y` / `n`：确认或取消 kill
- `?` / `h`：打开或关闭帮助
- `Esc`：退出聚焦模式或关闭弹层
- `q`：退出

TUI 必须运行在交互式终端里。

## JSON 输出

`list`、`info`、`find` 支持 `--json`。

它的设计目标是：

- 字段名稳定，方便自动化处理
- 文本输出和 JSON 输出复用同一套服务层数据
- 尽量保留部分信息，而不是因为个别字段失败就让整个命令报错

当某个值无法采集时：

- 文本输出显示 `N/A`
- JSON 输出使用 `null`

这也让 JSON 输出很适合后续封装成编辑器集成、shell 工作流或更高层的自动化接口。

## 架构概览

项目当前大致分为三层：

- `src/platform`：平台相关的 socket 和进程采集
- `src/core`：共享模型、scope 判定、warning 规则、服务逻辑
- `src/output` 和 `src/tui`：CLI 文本输出、JSON 输出、格式化和终端界面

这样做的目的是把“数据采集”和“呈现方式”分开，后续无论扩平台还是扩输出形式，维护成本都会更低。

## 开发

开发过程中常用命令：

```bash
cargo fmt --check
cargo clippy
cargo test
```

另见：

- [CHANGELOG.md](./CHANGELOG.md)
- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [RELEASING.md](./RELEASING.md)
- [LICENSE](./LICENSE)

## 当前限制

- 目前只支持 macOS 和 Linux
- Windows 还不在当前实现范围内
- 某些进程字段依赖系统权限
- TUI 目前仍然是轻量版本，后续还会继续打磨

## Roadmap

- Windows 支持
- 更好的 TUI 导航和详情体验
- 更强的交互式筛选和排序能力
- 更完善的打包和发布流程
- 基于 JSON 输出的更高层集成能力
