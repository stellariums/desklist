# Desklist 本机 Agent API

Desklist 提供仅监听 `127.0.0.1` 的 Agent REST API 和 Streamable HTTP MCP，让 Codex、Claude、Hermes 等本机工具查询和管理任务。

## 获取访问令牌

1. 启动 Desklist。
2. 打开“设置”。
3. 在“AI / Agent 接入”中复制接口地址和访问令牌。
4. 将 Agent 名称作为 `X-Desklist-Agent` 请求头发送，Desklist 会把它记录到审计日志。

令牌等同于本机 Desklist 的操作权限，不要把它写入公开仓库、聊天截图或日志。怀疑泄露时，在设置中点击“重新生成令牌”，旧令牌会立即失效。

## 连接 Codex

Desklist 必须保持运行。打开 Codex 的“设置 → MCP servers”，选择“Add server”：

1. 名称填写 `desklist`。
2. 类型选择 `Streamable HTTP`。
3. 地址填写设置页显示的 MCP 地址：`http://127.0.0.1:47831/mcp`。
4. Bearer token 填写 Desklist 设置页显示的访问令牌。
5. 保存并按 Codex 提示重启本机客户端。

也可以通过 Codex 的 `config.toml` 配置。令牌必须从环境变量读取，不要直接提交到配置示例或仓库：

```toml
[mcp_servers.desklist]
url = "http://127.0.0.1:47831/mcp"
bearer_token_env_var = "DESKLIST_AGENT_TOKEN"
default_tools_approval_mode = "writes"
```

重新启动 Codex 后，可以在 `/mcp` 中确认 `desklist` 已连接。之后可直接说“把整理课程笔记放进 Desklist 收件箱”或“明天下午三点提醒我提交报告”。

Desklist MCP 提供以下工具：

- `list_tasks`：查询任务
- `get_task`：按 ID 查询单条任务，避免为了修改或验收读取整个列表
- `capture_task`：把没有确定时间的事项放进收件箱
- `create_task`：新建有明确安排时间的任务
- `update_task`：修改任务
- `complete_task` / `reopen_task`：完成或恢复未完成
- `trash_task` / `restore_task`：移入回收站或从回收站恢复

Codex MCP 配置方式以[官方 OpenAI MCP 文档](https://developers.openai.com/codex/mcp/)为准。

## 请求格式

所有 Agent API 请求都需要以下请求头：

```text
Authorization: Bearer <Desklist 访问令牌>
X-Desklist-Agent: codex-local
```

接口根地址：

```text
http://127.0.0.1:47831/api/agent/v1
```

OpenAPI 文档可在 Desklist 运行时直接访问：

```text
http://127.0.0.1:47831/api/agent/v1/openapi.json
```

## PowerShell 示例

先把从设置页复制的令牌放入当前终端变量：

```powershell
$desklistToken = '<从 Desklist 设置页复制的令牌>'
$desklistHeaders = @{
  Authorization = "Bearer $desklistToken"
  'X-Desklist-Agent' = 'codex-local'
}
```

查询未完成任务：

```powershell
Invoke-RestMethod `
  -Uri 'http://127.0.0.1:47831/api/agent/v1/events?filter=upcoming' `
  -Headers $desklistHeaders
```

创建有安排时间的任务：

```powershell
$desklistBody = @{
  title = '提交报告'
  description = '发送最终版本'
  event_time = '2026-09-01T15:00:00+08:00'
  scheduled_end = $null
  due_time = '2026-09-01T15:00:00+08:00'
  remind_at = '2026-09-01T14:45:00+08:00'
  remind_on_time = 1
  recurrence = 'none'
  recurrence_end = $null
} | ConvertTo-Json

Invoke-RestMethod `
  -Method Post `
  -Uri 'http://127.0.0.1:47831/api/agent/v1/events' `
  -Headers $desklistHeaders `
  -ContentType 'application/json' `
  -Body $desklistBody
```

没有确定时间的任务应进入收件箱：

```powershell
$desklistInboxBody = @{
  title = '整理课程笔记'
  description = '稍后再安排时间'
} | ConvertTo-Json

Invoke-RestMethod `
  -Method Post `
  -Uri 'http://127.0.0.1:47831/api/agent/v1/inbox' `
  -Headers $desklistHeaders `
  -ContentType 'application/json' `
  -Body $desklistInboxBody
```

完成任务：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri 'http://127.0.0.1:47831/api/agent/v1/events/<任务 ID>/complete' `
  -Headers $desklistHeaders
```

`complete` 和 `reopen` 是幂等操作：Agent 因网络问题重复请求，不会把任务反向切换，也不会重复生成周期任务。

## 删除边界

Agent API 不提供永久删除。`POST /events/{id}/trash` 只会把任务移入回收站，使用 `POST /events/{id}/restore` 可以恢复。

## 安全边界

- 服务只监听本机地址，不支持局域网或公网访问。
- Agent API 使用独立 Bearer 令牌；令牌重新生成后旧值立即失效。
- MCP 查询工具标记为只读，写工具会按 Codex 的 `writes` 审批策略请求确认，回收站操作标记为破坏性操作。
- 每次 Agent 写操作都会记录任务 ID、操作、来源和时间。
- 令牌不能隔离同一 Windows 账户下已经取得用户权限的恶意程序；它的作用是限定可信 Agent 的正式调用入口并提供可追踪性。
