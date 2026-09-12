# 《AIPCMaster（AI电脑大师）实体关系图（ERD）单独版》

**文档编号：** AIPCMaster-ERD-2026-09-V1.0  
**版本：** V1.0 最终版  
**日期：** 2026年9月12日  
**英文产品名：** AIPCMaster  
**中文产品名：** AI电脑大师  
**正式组合写法：** AIPCMaster（AI电脑大师）  
**主域名：** aipcmaster.com  
**官网：** [https://aipcmaster.com](https://aipcmaster.com/)  
**开发者邮箱：** [dev@aipcmaster.com](mailto:dev@aipcmaster.com)  
**版权署名：** © 2026 AIPCMaster. All rights reserved.

**保密声明：**  
本文档包含 AIPCMaster（AI电脑大师）数据库设计、实体关系与数据隐私策略等商业秘密，仅供内部研发、架构评审与合作洽谈使用。未经书面许可，不得向无关第三方披露、复制或传播。


## 1. 设计原则

- 用户、组织、设备、订阅、诊断、优化、预测、审计为核心域。

- 个人版与企业版共用核心模型，通过 organization\_id 区分。

- 所有系统修改操作必须记录审计日志。

- 诊断数据默认本地存储，云端仅存储脱敏摘要。

- 敏感字段加密存储。

- 支持软删除、版本化、审计追踪。


## 2. 核心实体关系图

```
erDiagram  
    USER ||--o\{ DEVICE : owns  
    USER ||--o\{ SUBSCRIPTION : has  
    USER ||--o\{ ORDER : places  
    USER ||--o\{ SUPPORT\_TICKET : creates  
    USER ||--o\{ AUDIT\_LOG : acts  
    USER ||--o\{ API\_KEY : owns  
    USER ||--o\{ ALERT : receives  
  
    ORGANIZATION ||--o\{ ORGANIZATION\_MEMBER : contains  
    USER ||--o\{ ORGANIZATION\_MEMBER : joins  
    ORGANIZATION ||--o\{ DEVICE : manages  
    ORGANIZATION ||--o\{ SUBSCRIPTION : has  
    ORGANIZATION ||--o\{ ORDER : places  
    ORGANIZATION ||--o\{ API\_KEY : owns  
    ORGANIZATION ||--o\{ AUDIT\_LOG : records  
    ORGANIZATION ||--o\{ ALERT : receives  
  
    SUBSCRIPTION\_PLAN ||--o\{ SUBSCRIPTION : defines  
    SUBSCRIPTION ||--o\{ ORDER : paid\_by  
  
    DEVICE ||--o\{ DIAGNOSTIC\_SESSION : runs  
    DIAGNOSTIC\_SESSION ||--o\{ DIAGNOSTIC\_REPORT : generates  
    DIAGNOSTIC\_REPORT ||--o\{ ISSUE : contains  
    ISSUE ||--o\{ OPTIMIZATION\_ACTION : suggests  
    OPTIMIZATION\_ACTION ||--o\{ OPTIMIZATION\_LOG : logs  
    DEVICE ||--o\{ PREDICTION : predicts  
    DEVICE ||--o\{ ALERT : triggers
```


## 3. 核心表结构说明

### 3.1 users

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| email | VARCHAR | 邮箱，唯一 |
| phone | VARCHAR | 手机号，可空 |
| password\_hash | VARCHAR | 密码哈希 |
| nickname | VARCHAR | 昵称 |
| avatar\_url | VARCHAR | 头像 |
| status | VARCHAR | active/disabled/deleted |
| locale | VARCHAR | 语言 |
| two\_factor\_enabled | BOOLEAN | 两步验证 |
| created\_at | TIMESTAMP | 创建时间 |
| updated\_at | TIMESTAMP | 更新时间 |
| last\_login\_at | TIMESTAMP | 最后登录 |


### 3.2 organizations

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| name | VARCHAR | 组织名称 |
| owner\_user\_id | UUID | 所有者 |
| plan\_code | VARCHAR | 企业版套餐 |
| status | VARCHAR | active/suspended |
| created\_at | TIMESTAMP | 创建时间 |
| updated\_at | TIMESTAMP | 更新时间 |


### 3.3 organization\_members

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| org\_id | UUID | 组织 |
| user\_id | UUID | 用户 |
| role | VARCHAR | admin/member/viewer |
| joined\_at | TIMESTAMP | 加入时间 |


### 3.4 devices

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| user\_id | UUID | 所属用户，可空 |
| org\_id | UUID | 所属组织，可空 |
| device\_uuid | VARCHAR | 设备唯一标识 |
| hostname | VARCHAR | 主机名 |
| os | VARCHAR | Windows/macOS |
| os\_version | VARCHAR | 系统版本 |
| cpu | VARCHAR | CPU 信息 |
| memory\_mb | INT | 内存 |
| disk\_json | JSONB | 磁盘信息 |
| gpu | VARCHAR | GPU |
| npu | VARCHAR | NPU |
| last\_seen\_at | TIMESTAMP | 最后在线 |
| status | VARCHAR | online/offline/disabled |
| created\_at | TIMESTAMP | 创建时间 |


### 3.5 subscription\_plans

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| code | VARCHAR | trial/personal/family/business |
| name | VARCHAR | 套餐名 |
| price\_cents | INT | 价格，分 |
| currency | VARCHAR | CNY |
| period | VARCHAR | yearly |
| features\_json | JSONB | 权益 |
| active | BOOLEAN | 是否上架 |


### 3.6 subscriptions

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| user\_id | UUID | 用户，可空 |
| org\_id | UUID | 组织，可空 |
| plan\_id | UUID | 套餐 |
| status | VARCHAR | trialing/active/expired/canceled |
| start\_at | TIMESTAMP | 开始 |
| end\_at | TIMESTAMP | 结束 |
| trial\_end\_at | TIMESTAMP | 试用结束 |
| auto\_renew | BOOLEAN | 自动续费 |
| created\_at | TIMESTAMP | 创建时间 |


### 3.7 orders

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| user\_id | UUID | 用户，可空 |
| org\_id | UUID | 组织，可空 |
| subscription\_id | UUID | 订阅 |
| amount\_cents | INT | 金额 |
| currency | VARCHAR | CNY |
| status | VARCHAR | pending/paid/failed/refunded |
| payment\_provider | VARCHAR | alipay/wechat/stripe |
| paid\_at | TIMESTAMP | 支付时间 |
| created\_at | TIMESTAMP | 创建时间 |


### 3.8 diagnostic\_sessions

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| device\_id | UUID | 设备 |
| user\_id | UUID | 用户 |
| trigger\_type | VARCHAR | manual/scheduled/alert |
| status | VARCHAR | running/completed/failed |
| started\_at | TIMESTAMP | 开始 |
| ended\_at | TIMESTAMP | 结束 |
| summary | TEXT | 摘要 |


### 3.9 diagnostic\_reports

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| session\_id | UUID | 诊断会话 |
| device\_id | UUID | 设备 |
| health\_score | INT | 健康分 |
| summary | TEXT | 摘要 |
| details\_json | JSONB | 详情 |
| created\_at | TIMESTAMP | 创建时间 |


### 3.10 issues

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| report\_id | UUID | 报告 |
| device\_id | UUID | 设备 |
| category | VARCHAR | performance/disk/memory/security |
| severity | VARCHAR | low/medium/high/critical |
| title | VARCHAR | 标题 |
| description | TEXT | 描述 |
| evidence\_json | JSONB | 证据 |
| status | VARCHAR | open/resolved/ignored |
| detected\_at | TIMESTAMP | 检测时间 |


### 3.11 optimization\_actions

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| issue\_id | UUID | 问题 |
| device\_id | UUID | 设备 |
| action\_type | VARCHAR | clean\_memory/startup/disk |
| risk\_level | VARCHAR | low/medium/high |
| requires\_confirm | BOOLEAN | 是否需要确认 |
| parameters\_json | JSONB | 参数 |
| status | VARCHAR | suggested/approved/executed/rolled\_back |
| created\_at | TIMESTAMP | 创建时间 |


### 3.12 optimization\_logs

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| action\_id | UUID | 操作 |
| device\_id | UUID | 设备 |
| user\_id | UUID | 用户 |
| executed\_at | TIMESTAMP | 执行时间 |
| result | VARCHAR | success/failed |
| rollback\_status | VARCHAR | none/success/failed |
| log\_text | TEXT | 日志 |


### 3.13 predictions

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| device\_id | UUID | 设备 |
| model\_version | VARCHAR | 模型版本 |
| prediction\_type | VARCHAR | disk/memory/driver |
| probability | DECIMAL | 概率 |
| horizon\_days | INT | 预测天数 |
| details\_json | JSONB | 详情 |
| created\_at | TIMESTAMP | 创建时间 |


### 3.14 alerts

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| device\_id | UUID | 设备 |
| user\_id | UUID | 用户，可空 |
| org\_id | UUID | 组织，可空 |
| type | VARCHAR | performance/security/subscription |
| severity | VARCHAR | low/medium/high/critical |
| title | VARCHAR | 标题 |
| message | TEXT | 内容 |
| status | VARCHAR | unread/read/resolved |
| created\_at | TIMESTAMP | 创建时间 |


### 3.15 audit\_logs

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| actor\_user\_id | UUID | 操作者 |
| org\_id | UUID | 组织，可空 |
| action | VARCHAR | 操作 |
| resource\_type | VARCHAR | 资源类型 |
| resource\_id | UUID | 资源 ID |
| ip | VARCHAR | IP |
| user\_agent | VARCHAR | UA |
| created\_at | TIMESTAMP | 创建时间 |


### 3.16 api\_keys

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| user\_id | UUID | 用户，可空 |
| org\_id | UUID | 组织，可空 |
| name | VARCHAR | 名称 |
| key\_hash | VARCHAR | 密钥哈希 |
| scopes | JSONB | 权限范围 |
| last\_used\_at | TIMESTAMP | 最后使用 |
| expires\_at | TIMESTAMP | 过期时间 |
| status | VARCHAR | active/revoked |


### 3.17 support\_tickets

| 字段 | 类型 | 说明 |
| - | - | - |
| id | UUID | 主键 |
| user\_id | UUID | 用户 |
| org\_id | UUID | 组织，可空 |
| device\_id | UUID | 设备，可空 |
| subject | VARCHAR | 主题 |
| content | TEXT | 内容 |
| status | VARCHAR | open/pending/resolved/closed |
| priority | VARCHAR | low/normal/high/urgent |
| created\_at | TIMESTAMP | 创建时间 |



## 4. 索引与约束建议

- users.email 唯一索引

- devices.device\_uuid 唯一索引

- subscriptions.user\_id + status 联合索引

- subscriptions.org\_id + status 联合索引

- diagnostic\_sessions.device\_id + started\_at 联合索引

- diagnostic\_reports.session\_id 唯一索引

- issues.report\_id + severity 联合索引

- optimization\_logs.action\_id 唯一索引

- predictions.device\_id + created\_at 联合索引

- alerts.user\_id + status 联合索引

- audit\_logs.org\_id + created\_at 联合索引

- api\_keys.key\_hash 唯一索引


## 5. 数据保留与隐私

- 诊断原始数据默认本地存储。

- 云端仅存储脱敏摘要、健康分、问题类型。

- 审计日志保留不少于 180 天。

- 用户可申请导出和删除个人数据。

- 敏感字段加密存储。

- 密钥使用 KMS/Vault 管理。

- 符合 GDPR、个人信息保护法、数据安全法。


## 6. 联系信息

**AIPCMaster（AI电脑大师）**  
官网：https://aipcmaster.com  
开发者邮箱：dev@aipcmaster.com  
安全邮箱：security@aipcmaster.com  
支持邮箱：support@aipcmaster.com  
© 2026 AIPCMaster. All rights reserved.
