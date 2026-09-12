# AIPCMaster（AI电脑大师）-核心时序图

**适用文档：** 《AIPCMaster（AI电脑大师）软件开发工程文档单独版》V1.0  
**英文产品名：** AIPCMaster  
**中文产品名：** AI电脑大师  
**正式组合写法：** AIPCMaster（AI电脑大师）  
**主域名：** aipcmaster.com  
**官网：** [https://aipcmaster.com](https://aipcmaster.com/)  
**开发者邮箱：** [dev@aipcmaster.com](mailto:dev@aipcmaster.com)  
**安全邮箱：** [security@aipcmaster.com](mailto:security@aipcmaster.com)  
**版权署名：** © 2026 AIPCMaster. All rights reserved.

**插入位置建议：**  
将以下内容插入《AIPCMaster（AI电脑大师）软件开发工程文档单独版》第 4 章“API 设计”之后，作为新的 **第 5 章 核心时序图**，原第 5 章及以后章节顺延。  
其中 5.1、5.3、5.4 也可复用到《PRD 单独版》的“核心用户流程”章节。

## 5. 核心时序图

### 5.1 注册、试用开通与包年转化

```
sequenceDiagram    
    autonumber    
    actor U as 用户    
    participant C as AIPCMaster客户端    
    participant G as API网关    
    participant A as 认证服务    
    participant S as 订阅服务    
    participant P as 支付服务    
    participant D as PostgreSQL    
    participant M as 邮件服务    
    
    U-\\\>\\\>C: 输入邮箱/密码注册    
    C-\\\>\\\>G: POST /api/v1/auth/register    
    G-\\\>\\\>A: 创建账号    
    A-\\\>\\\>D: 写入 users    
    A-\\\>\\\>S: 创建14天试用订阅    
    S-\\\>\\\>D: 写入 subscriptions(trialing)    
    A-\\\>\\\>M: 发送验证邮件    
    A--\\\>\\\>G: 返回 JWT + Refresh Token    
    G--\\\>\\\>C: 注册成功，试用已开通    
    C--\\\>\\\>U: 展示全功能试用    
    
    Note over S,M: 试用到期前3天/1天提醒    
    S-\\\>\\\>M: 发送到期提醒    
    M--\\\>\\\>U: 邮件提醒    
    
    U-\\\>\\\>C: 选择个人版 ¥199/年    
    C-\\\>\\\>G: POST /api/v1/subscriptions/checkout    
    G-\\\>\\\>S: 创建订单    
    S-\\\>\\\>D: 写入 orders(pending)    
    S-\\\>\\\>P: 创建支付单    
    P--\\\>\\\>U: 支付宝/微信支付    
    U-\\\>\\\>P: 完成支付    
    P-\\\>\\\>G: 支付回调    
    G-\\\>\\\>S: 更新订单为 paid    
    S-\\\>\\\>D: 更新 subscriptions(active, end\\\_at=+1年)    
    S-\\\>\\\>M: 发送收据/发票    
    S--\\\>\\\>C: 订阅已生效
```

### 5.2 设备注册与绑定

```
sequenceDiagram    
    autonumber    
    actor U as 用户    
    participant C as AIPCMaster客户端    
    participant G as API网关    
    participant Dev as 设备服务    
    participant D as PostgreSQL    
    participant Audit as 审计服务    
    
    C-\\\>\\\>C: 首次启动生成设备唯一标识 device\\\_uuid    
    U-\\\>\\\>C: 登录账号    
    C-\\\>\\\>G: POST /api/v1/devices/register    
    G-\\\>\\\>Dev: 校验用户身份与设备数量权限    
    Dev-\\\>\\\>D: 写入 devices(user\\\_id, device\\\_uuid, hostname, os, cpu, memory)    
    Dev-\\\>\\\>Audit: 写入设备绑定审计日志    
    Dev--\\\>\\\>G: 返回设备ID与绑定状态    
    G--\\\>\\\>C: 设备绑定成功    
    C--\\\>\\\>U: 展示设备已绑定，可开始诊断
```

### 5.3 智能诊断

```
sequenceDiagram    
    autonumber    
    actor U as 用户    
    participant C as AIPCMaster客户端    
    participant S as 数据采集层    
    participant AI as 本地AI推理引擎    
    participant T as 工具调用路由    
    participant Cloud as 云端诊断服务    
    participant D as 本地存储    
    
    U-\\\>\\\>C: 点击“一键诊断”或输入自然语言    
    C-\\\>\\\>S: 采集CPU/内存/磁盘/网络/温度/进程    
    S--\\\>\\\>AI: 系统指标与时序数据    
    AI-\\\>\\\>AI: 异常检测与根因分析    
    AI-\\\>\\\>T: 调用系统诊断工具    
    T--\\\>\\\>AI: 返回进程、启动项、磁盘、驱动数据    
    AI-\\\>\\\>D: 写入本地诊断会话与报告    
    AI--\\\>\\\>C: 生成健康分、问题、证据、建议    
    C--\\\>\\\>U: 展示自然语言诊断报告    
    
    opt 用户授权云端深度分析    
        C-\\\>\\\>Cloud: 上传脱敏统计摘要    
        Cloud-\\\>\\\>Cloud: 跨时间段/多设备对比分析    
        Cloud--\\\>\\\>C: 返回增强分析结果    
        C--\\\>\\\>U: 展示云端增强报告    
    end
```

### 5.4 自动优化与回滚

```
sequenceDiagram    
    autonumber    
    actor U as 用户    
    participant C as AIPCMaster客户端    
    participant O as 优化执行模块    
    participant R as 系统还原/回滚模块    
    participant L as 本地审计日志    
    participant OS as 操作系统    
    
    C--\\\>\\\>U: 展示优化建议与风险等级    
    U-\\\>\\\>C: 选择执行优化    
    C-\\\>\\\>O: 提交 action\\\_id 与参数    
    O-\\\>\\\>R: 创建系统还原点    
    R--\\\>\\\>O: 还原点创建成功    
    O-\\\>\\\>OS: 执行内存释放/启动项/磁盘清理    
    OS--\\\>\\\>O: 返回执行结果    
    alt 执行成功    
        O-\\\>\\\>L: 写入成功日志    
        O--\\\>\\\>C: 优化完成    
        C--\\\>\\\>U: 展示优化结果    
    else 执行失败或用户不满意    
        O-\\\>\\\>R: 触发一键回滚    
        R-\\\>\\\>OS: 恢复系统还原点    
        R-\\\>\\\>L: 写入回滚日志    
        R--\\\>\\\>C: 回滚完成    
        C--\\\>\\\>U: 展示回滚结果    
    end
```

### 5.5 企业批量策略下发

```
sequenceDiagram    
    autonumber    
    actor Admin as 企业管理员    
    participant Console as 企业控制台    
    participant Org as 组织服务    
    participant Policy as 策略服务    
    participant GW as API网关    
    participant Agent as 企业设备客户端    
    participant Audit as 审计服务    
    
    Admin-\\\>\\\>Console: 创建批量优化策略    
    Console-\\\>\\\>GW: POST /api/v1/orgs/\\\{orgId\\\}/policies    
    GW-\\\>\\\>Org: 校验管理员RBAC    
    Org-\\\>\\\>Policy: 保存策略    
    Policy--\\\>\\\>Console: 返回策略ID    
    Admin-\\\>\\\>Console: 选择设备组并下发    
    Console-\\\>\\\>Policy: 下发策略    
    Policy-\\\>\\\>Agent: 通过WebSocket/长连接推送    
    Agent-\\\>\\\>Agent: 本地校验权限与风险    
    Agent--\\\>\\\>Policy: 上报执行结果    
    Policy-\\\>\\\>Audit: 写入审计日志    
    Policy--\\\>\\\>Console: 更新执行状态    
    Console--\\\>\\\>Admin: 展示策略执行报告
```

### 5.6 API Key 调用与审计

```
sequenceDiagram    
    autonumber    
    actor Dev as 开发者    
    participant GW as API网关    
    participant Auth as 认证服务    
    participant Biz as 业务服务    
    participant Audit as 审计服务    
    participant D as PostgreSQL    
    
    Dev-\\\>\\\>GW: POST /api/v1/api-keys (JWT)    
    GW-\\\>\\\>Auth: 校验用户/组织权限    
    Auth-\\\>\\\>D: 写入 api\\\_keys(key\\\_hash, scopes)    
    Auth--\\\>\\\>Dev: 返回一次性明文API Key    
    
    Dev-\\\>\\\>GW: 携带 API Key 调用 /api/v1/devices    
    GW-\\\>\\\>Auth: 校验 key\\\_hash、scopes、过期时间    
    Auth-\\\>\\\>Auth: 限流与防重放校验    
    Auth--\\\>\\\>GW: 认证通过    
    GW-\\\>\\\>Biz: 查询设备数据    
    Biz--\\\>\\\>GW: 返回数据    
    GW-\\\>\\\>Audit: 写入审计日志    
    GW--\\\>\\\>Dev: 返回统一响应
```

### 5.7 预测性维护告警

```
sequenceDiagram    
    autonumber    
    participant Agent as AIPCMaster客户端    
    participant AI as 本地预测模型    
    participant Cloud as 预测服务    
    participant Alert as 告警服务    
    participant Notify as 通知服务    
    actor U as 用户    
    
    Agent-\\\>\\\>AI: 输入CPU/内存/磁盘/温度时序    
    AI-\\\>\\\>AI: LSTM+Attention推理    
    AI--\\\>\\\>Agent: 未来7天风险概率    
    Agent-\\\>\\\>Cloud: 上传脱敏风险摘要    
    Cloud-\\\>\\\>Alert: 创建告警    
    Alert-\\\>\\\>Notify: 触发邮件/应用内通知    
    Notify--\\\>\\\>U: 硬盘/内存/驱动风险提醒    
    U-\\\>\\\>Agent: 查看维护建议    
    Agent--\\\>\\\>U: 展示修复/备份/更换建议
```

### 5.8 工单与开发者支持流程

```
sequenceDiagram    
    autonumber    
    actor U as 用户/开发者    
    participant C as 客户端/官网    
    participant GW as API网关    
    participant Ticket as 工单服务    
    participant Support as 支持人员    
    participant Dev as 开发者支持    
    participant M as 邮件服务    
    
    U-\\\>\\\>C: 提交工单或反馈    
    C-\\\>\\\>GW: POST /api/v1/support/tickets    
    GW-\\\>\\\>Ticket: 创建工单    
    Ticket-\\\>\\\>M: 通知 support@aipcmaster.com    
    alt 普通用户问题    
        Support-\\\>\\\>Ticket: 处理并回复    
        Ticket--\\\>\\\>U: 邮件/应用内通知    
    else 开发者/API问题    
        Ticket-\\\>\\\>M: 转交 dev@aipcmaster.com    
        Dev-\\\>\\\>Ticket: 技术排查与回复    
        Ticket--\\\>\\\>U: 返回技术解决方案    
    end
```

## 6. 文档更新说明

1. 《AIPCMaster（AI电脑大师）软件开发工程文档单独版》版本由 V1.0 升至 **V1.1**。

2. 新增 **第 5 章 核心时序图**，原第 5 章及以后章节顺延。

3. 开发者邮箱统一为：\*\*[dev@aipcmaster.com\*\*](mailto:dev@aipcmaster.com)。

4. 安全漏洞报告邮箱统一为：\*\*[security@aipcmaster.com\*\*](mailto:security@aipcmaster.com)。

5. 《PRD 单独版》可复用 5.1、5.3、5.4 作为用户核心流程时序图。

6. 《ERD 单独版》不涉及时序图，无需调整。

**AIPCMaster（AI电脑大师）**  
官网：https://aipcmaster.com  
开发者邮箱：dev@aipcmaster.com  
安全邮箱：security@aipcmaster.com  
支持邮箱：support@aipcmaster.com  
销售邮箱：sales@aipcmaster.com  
商务合作：business@aipcmaster.com  
© 2026 AIPCMaster. All rights reserved.

