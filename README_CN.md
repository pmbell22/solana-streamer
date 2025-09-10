# Solana Streamer
[中文](https://github.com/0xfnzero/solana-streamer/blob/main/README_CN.md) | [English](https://github.com/0xfnzero/solana-streamer/blob/main/README.md) | [Telegram](https://t.me/fnzero_group)

一个轻量级的 Rust 库，用于从 Solana DEX 交易程序中实时流式传输事件。该库为 PumpFun、PumpSwap、Bonk 和 Raydium CPMM 协议提供高效的事件解析和订阅功能。

## 项目特性

1. **实时事件流**: 订阅多个 Solana DEX 协议的实时交易事件
2. **Yellowstone gRPC 支持**: 使用 Yellowstone gRPC 进行高性能事件订阅
3. **ShredStream 支持**: 使用 ShredStream 协议进行替代事件流传输
4. **多协议支持**: 
   - **PumpFun**: 迷因币交易平台事件
   - **PumpSwap**: PumpFun 的交换协议事件
   - **Bonk**: 代币发布平台事件 (letsbonk.fun)
   - **Raydium CPMM**: Raydium 集中池做市商事件
   - **Raydium CLMM**: Raydium 集中流动性做市商事件
   - **Raydium AMM V4**: Raydium 自动做市商 V4 事件
5. **统一事件接口**: 在所有支持的协议中保持一致的事件处理
6. **事件解析系统**: 自动解析和分类协议特定事件
7. **账户状态监控**: 实时监控协议账户状态和配置变更
8. **交易与账户事件过滤**: 分别过滤交易事件和账户状态变化
9. **高性能**: 针对低延迟事件处理进行优化
10. **批处理优化**: 批量处理事件以减少回调开销
11. **性能监控**: 内置性能指标监控，包括事件处理速度等
12. **内存优化**: 对象池和缓存机制减少内存分配
13. **灵活配置系统**: 支持自定义批处理大小、背压策略、通道大小等参数
14. **预设配置**: 提供高吞吐量、低延迟等预设配置，针对不同使用场景优化
15. **背压处理**: 支持阻塞、丢弃等背压策略
16. **运行时配置更新**: 支持在运行时动态更新配置参数
17. **全函数性能监控**: 所有subscribe_events函数都支持性能监控，自动收集和报告性能指标
18. **优雅关闭**: 支持编程式 stop() 方法进行干净的关闭
19. **动态订阅管理**: 运行时过滤器更新而无需重新连接，支持自适应监控策略

## 安装

### 直接克隆

将项目克隆到您的项目目录：

```bash
cd your_project_root_directory
git clone https://github.com/0xfnzero/solana-streamer
```

在您的 `Cargo.toml` 中添加依赖：

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = { path = "./solana-streamer", version = "0.4.5" }
```

### 使用 crates.io

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = "0.4.5"
```

## 配置系统

### 预设配置

库提供了三种预设配置，针对不同的使用场景进行了优化：

#### 1. 高吞吐量配置 (`high_throughput()`)

专为高并发场景优化，优先考虑吞吐量而非延迟：

```rust
let config = StreamClientConfig::high_throughput();
// 或者使用便捷方法
let grpc = YellowstoneGrpc::new_high_throughput(endpoint, token)?;
let shred = ShredStreamGrpc::new_high_throughput(endpoint).await?;
```

**特性：**
- **背压策略**: Drop（丢弃策略）- 在高负载时丢弃消息以避免阻塞
- **缓冲区大小**: 5,000 个许可证，处理突发流量
- **适用场景**: 需要处理大量数据且可以容忍在峰值负载时偶尔丢失消息的场景

#### 2. 低延迟配置 (`low_latency()`)

专为实时场景优化，优先考虑延迟而非吞吐量：

```rust
let config = StreamClientConfig::low_latency();
// 或者使用便捷方法
let grpc = YellowstoneGrpc::new_low_latency(endpoint, token)?;
let shred = ShredStreamGrpc::new_low_latency(endpoint).await?;
```

**特性：**
- **背压策略**: Block（阻塞策略）- 确保不丢失任何数据
- **缓冲区大小**: 4000 个许可证，平衡吞吐量和延迟
- **立即处理**: 不进行缓冲，立即处理事件
- **适用场景**: 每毫秒都很重要且不能丢失任何事件的场景，如交易应用或实时监控


### 自定义配置

您也可以创建自定义配置：

```rust
let config = StreamClientConfig {
    connection: ConnectionConfig {
        connect_timeout: 30,
        request_timeout: 120,
        max_decoding_message_size: 20 * 1024 * 1024, // 20MB
    },
    backpressure: BackpressureConfig {
        permits: 2000,
        strategy: BackpressureStrategy::Block,
    },
    enable_metrics: true,
};
```

## 使用示例

### 使用示例概览表

| 功能类型 | 示例文件 | 描述 | 运行命令 | 源码路径 |
|---------|---------|------|---------|----------|
| Yellowstone gRPC 流 | `grpc_example.rs` | 使用 Yellowstone gRPC 监控交易事件 | `cargo run --example grpc_example` | [examples/grpc_example.rs](examples/grpc_example.rs) |
| ShredStream 流 | `shred_example.rs` | 使用 ShredStream 监控交易事件 | `cargo run --example shred_example` | [examples/shred_example.rs](examples/shred_example.rs) |
| 解析交易事件 | `parse_tx_events` | 解析 Solana 主网交易数据 | `cargo run --example parse_tx_events` | [examples/parse_tx_events.rs](examples/parse_tx_events.rs) |
| 动态订阅管理 | `dynamic_subscription` | 运行时更新过滤器 | `cargo run --example dynamic_subscription` | [examples/dynamic_subscription.rs](examples/dynamic_subscription.rs) |
| 代币余额监控 | `token_balance_listen_example` | 监控特定代币账户余额变化 | `cargo run --example token_balance_listen_example` | [examples/token_balance_listen_example.rs](examples/token_balance_listen_example.rs) |
| Nonce 账户监控 | `nonce_listen_example` | 跟踪 nonce 账户状态变化 | `cargo run --example nonce_listen_example` | [examples/nonce_listen_example.rs](examples/nonce_listen_example.rs) |

### 事件过滤

库支持灵活的事件过滤以减少处理开销并提升性能：

#### 基础过滤

```rust
use solana_streamer_sdk::streaming::event_parser::common::{filter::EventTypeFilter, EventType};

// 无过滤 - 接收所有事件
let event_type_filter = None;

// 过滤特定事件类型 - 只接收 PumpSwap 买入/卖出事件
let event_type_filter = Some(EventTypeFilter { 
    include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] 
});
```

#### 性能影响

事件过滤可以带来显著的性能提升：
- **减少 60-80%** 的不必要事件处理
- **降低内存使用** 通过过滤掉无关事件
- **减少网络带宽** 在分布式环境中
- **更好的专注性** 只处理对应用有意义的事件

#### 按使用场景的过滤示例

**交易机器人（专注交易事件）**
```rust
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapBuy,
        EventType::PumpSwapSell,
        EventType::PumpFunTrade,
        EventType::RaydiumCpmmSwap,
        EventType::RaydiumClmmSwap,
        EventType::RaydiumAmmV4Swap,
        .....
    ] 
});
```

**池监控（专注流动性事件）**
```rust
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapCreatePool,
        EventType::PumpSwapDeposit,
        EventType::PumpSwapWithdraw,
        EventType::RaydiumCpmmInitialize,
        EventType::RaydiumCpmmDeposit,
        EventType::RaydiumCpmmWithdraw,
        EventType::RaydiumClmmCreatePool,
        ......
    ] 
});
```

## 动态订阅管理

在运行时更新订阅过滤器而无需重新连接到流。

```rust
// 在现有订阅上更新过滤器
grpc.update_subscription(
    TransactionFilter {
        account_include: vec!["new_program_id".to_string()],
        account_exclude: vec![],
        account_required: vec![],
    },
    AccountFilter {
        account: vec![],
        owner: vec![],
    },
).await?;
```

- **无需重新连接**: 过滤器变更立即生效，无需关闭流
- **原子更新**: 交易和账户过滤器同时更新
- **单一订阅**: 每个客户端实例只有一个活跃订阅
- **兼容性**: 与立即订阅和高级订阅方法兼容

注意：在同一客户端上多次尝试订阅会返回错误。

## 支持的协议

- **PumpFun**: 主要迷因币交易平台
- **PumpSwap**: PumpFun 的交换协议
- **Bonk**: 代币发布平台 (letsbonk.fun)
- **Raydium CPMM**: Raydium 集中池做市商协议
- **Raydium CLMM**: Raydium 集中流动性做市商协议
- **Raydium AMM V4**: Raydium 自动做市商 V4 协议

## 事件流服务

- **Yellowstone gRPC**: 高性能 Solana 事件流
- **ShredStream**: 替代事件流协议

## 架构特性

### 统一事件接口

- **UnifiedEvent Trait**: 所有协议事件实现通用接口
- **Protocol Enum**: 轻松识别事件来源
- **Event Factory**: 自动事件解析和分类

### 事件解析系统

- **协议特定解析器**: 每个支持协议的专用解析器
- **事件工厂**: 集中式事件创建和解析
- **可扩展设计**: 易于添加新协议和事件类型

### 流基础设施

- **Yellowstone gRPC 客户端**: 针对 Solana 事件流优化
- **ShredStream 客户端**: 替代流实现
- **高性能处理**: 优化的事件处理机制

## 项目结构

```
src/
├── common/           # 通用功能和类型
├── protos/           # Protocol buffer 定义
├── streaming/        # 事件流系统
│   ├── event_parser/ # 事件解析系统
│   │   ├── common/   # 通用事件解析工具
│   │   ├── core/     # 核心解析特征和接口
│   │   ├── protocols/# 协议特定解析器
│   │   │   ├── bonk/ # Bonk 事件解析
│   │   │   ├── pumpfun/ # PumpFun 事件解析
│   │   │   ├── pumpswap/ # PumpSwap 事件解析
│   │   │   ├── raydium_amm_v4/ # Raydium AMM V4 事件解析
│   │   │   ├── raydium_cpmm/ # Raydium CPMM 事件解析
│   │   │   └── raydium_clmm/ # Raydium CLMM 事件解析
│   │   └── factory.rs # 解析器工厂
│   ├── shred_stream.rs # ShredStream 客户端
│   ├── yellowstone_grpc.rs # Yellowstone gRPC 客户端
│   └── yellowstone_sub_system.rs # Yellowstone 子系统
└── lib.rs            # 主库文件
```

## 性能考虑

1. **连接管理**: 正确处理连接生命周期和重连
2. **事件过滤**: 使用协议过滤减少不必要的事件处理
3. **内存管理**: 为长时间运行的流实现适当的清理
4. **错误处理**: 对网络问题和服务中断进行健壮的错误处理
5. **批处理优化**: 使用批处理减少回调开销，提高吞吐量
6. **性能监控**: 启用性能监控以识别瓶颈和优化机会
7. **优雅关闭**: 使用 stop() 方法进行干净关闭，并实现信号处理器以正确清理资源

## 许可证

MIT 许可证

## 联系方式

- 项目仓库: https://github.com/0xfnzero/solana-streamer
- Telegram 群组: https://t.me/fnzero_group

## 重要注意事项

1. **网络稳定性**: 确保稳定的网络连接以进行连续的事件流传输
2. **速率限制**: 注意公共 gRPC 端点的速率限制
3. **错误恢复**: 实现适当的错误处理和重连逻辑
5. **合规性**: 确保遵守相关法律法规

## 语言版本

- [English](README.md)
- [中文](README_CN.md)