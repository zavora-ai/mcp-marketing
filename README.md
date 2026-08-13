# Marketing Orchestration MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-marketing.svg)](https://crates.io/crates/mcp-marketing)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Multi-channel marketing orchestration for AI agents — campaigns, audiences, content generation, ads, social, email, performance analytics, attribution, and budget governance. 22 tools across the full campaign lifecycle.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-marketing/main/docs/assets/architecture.svg" alt="MCP Marketing Architecture" width="850"/>
</p>

## Why This Exists

The marketing MCP landscape is fragmented — single-platform tools (Google Ads MCP, MailerLite MCP) or read-only analytics. Nobody offers unified campaign orchestration across channels with budget governance.

| Capability | Competitors | mcp-marketing |
|-----------|:-----------:|:-------------:|
| Multi-channel orchestration | ❌ | ✅ |
| Campaign lifecycle (draft→launch→pause) | ❌ | ✅ |
| Budget governance (approval gates) | ❌ | ✅ |
| Cross-channel attribution | ⚠️ read-only | ✅ |
| Multi-backend (HubSpot, Mailchimp, Ads) | ❌ | ✅ |

## Tools (22)

### Campaigns (5)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_campaigns` | List by status/channel | read_only |
| `get_campaign` | Details + performance summary | read_only |
| `create_campaign` | Create multi-channel campaign (draft) | internal_write |
| `launch_campaign` | Activate across channels | gated (budget) |
| `pause_campaign` | Pause a running campaign | internal_write |

### Audiences (4)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_audiences` | List segments with sizes | read_only |
| `get_audience` | Segment definition + criteria | read_only |
| `create_audience` | Build segment from criteria | internal_write |
| `estimate_reach` | Estimate before committing | read_only |

### Content (4)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_content` | List assets | read_only |
| `create_content` | Generate copy (email, ad, social) | internal_write |
| `get_ab_variants` | A/B test variants + performance | read_only |
| `get_content_calendar` | Scheduled content view | read_only |

### Channels (4)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_channels` | Configured channels + status | read_only |
| `schedule_post` | Schedule social media post | external_write |
| `create_ad` | Create ad (Google/Meta/LinkedIn) | financial_action |
| `send_campaign_email` | Send email to audience | external_write |

### Performance (3)

| Tool | Purpose | Risk |
|------|---------|------|
| `get_campaign_metrics` | Impressions, clicks, conversions, ROAS | read_only |
| `get_attribution` | Multi-touch attribution | read_only |
| `get_channel_comparison` | Cross-channel performance | read_only |

### Budget (2)

| Tool | Purpose | Risk |
|------|---------|------|
| `get_budget_status` | Spend vs budget, burn rate | read_only |
| `allocate_budget` | Shift budget between channels | financial_action |

## Installation

```bash
cargo install mcp-marketing
```

## Configuration

| Backend | Env Vars | What it provides |
|---------|----------|-----------------|
| **HubSpot** | `HUBSPOT_ACCESS_TOKEN` | Campaigns, contacts, email, workflows |
| **Mailchimp** | `MAILCHIMP_API_KEY` | Campaigns, audiences, automations |
| **Custom API** | `MARKETING_API_URL` | Your own backend (full spec) |

Auto-detected from env vars. No data stored in the MCP server.

## Client Configuration

```json
{
  "mcpServers": {
    "marketing": {
      "command": "mcp-marketing",
      "args": [],
      "env": {
        "HUBSPOT_ACCESS_TOKEN": "your-token"
      }
    }
  }
}
```

## End-to-End Example: Product Launch Campaign

```
Agent: "Launch a product campaign for our new feature targeting enterprise users"

1. create_audience(name="Enterprise Users", criteria={"plan":"enterprise","active":true})
2. estimate_reach(criteria={"plan":"enterprise"}) → 2,400 users
3. create_campaign(name="Feature Launch", channels=["email","linkedin","google"], audience_id="aud-1", budget_cents=500000)
4. create_content(content_type="email_subject", brief="New feature announcement for enterprise")
5. create_content(content_type="ad_copy", brief="Enterprise feature - LinkedIn ad")
6. create_ad(platform="linkedin", campaign_id="camp-1", headline="...", body="...")
7. send_campaign_email(campaign_id="camp-1", audience_id="aud-1", subject="...", body="...")
8. launch_campaign(id="camp-1")
9. get_campaign_metrics(id="camp-1") → track performance
10. get_channel_comparison() → email vs linkedin vs google
11. allocate_budget(campaign_id="camp-1", allocations={"linkedin":300000,"google":200000})
```

## Governance

- `launch_campaign` — gated if budget exceeds threshold
- `create_ad` — commits ad spend (financial action)
- `send_campaign_email` — external write (customer-facing)
- `allocate_budget` — financial action, shifts real money

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.94.1 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.

## MCP 2026-07-28 rollout (P4 workflow/business)

This server uses `rmcp` 3.1.2 and `adk-mcp-sdk` 0.2 with a minimum supported
Rust version of **1.94.1**. It accepts stateless MCP 2026 requests with
per-request protocol, client identity, and capability metadata while retaining
the legacy MCP 2025-11-25 initialize flow for ordinary tools.

- **Tasks:** None; this server's operations are short-lived and execute directly.
- **MRTR approvals:** `send_campaign_email`
- **Discovery and routing:** rmcp serves on-demand discovery and validates the
  per-request protocol envelope; HTTP deployments can route with `Mcp-Method`
  and `Mcp-Name`. The packaged binary currently uses stdio.
- **Caching:** `tools/list` returns a public `ttlMs` of 60,000 for MCP 2026;
  rmcp omits the cache fields for legacy clients.
- **Deprecated extensions:** this server does not add new Roots, Sampling, or
  dynamic client-registration dependencies.

Protected tools require `MCP_REQUEST_STATE_KEY` with at least 32 high-entropy
bytes. All replicas must share that key so sealed approval state can resume on
another instance. Approval state is bound to the client identity, tool, and
arguments and expires after two minutes. Missing identity, invalid state,
rejection, or legacy protocol use fails closed. Task records are process-local
for the current stdio runtime; use a durable task store before deploying the
server behind scale-to-zero HTTP infrastructure.
