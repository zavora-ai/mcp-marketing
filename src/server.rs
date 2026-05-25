use crate::client::ApiClient;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde_json::json;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdInput { pub id: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FilterInput { pub status: Option<String>, pub channel: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateCampaignInput {
    pub name: String,
    pub channels: Vec<String>,
    pub audience_id: Option<String>,
    pub budget_cents: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateAudienceInput {
    pub name: String,
    pub criteria: serde_json::Value,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EstimateReachInput { pub criteria: serde_json::Value }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateContentInput {
    pub campaign_id: Option<String>,
    pub content_type: String, // "email_subject", "ad_copy", "social_post", "landing_page"
    pub brief: String,
    pub tone: Option<String>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SchedulePostInput {
    pub channel: String,
    pub content: String,
    pub scheduled_at: String,
    pub audience_id: Option<String>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateAdInput {
    pub platform: String, // "google", "meta", "linkedin"
    pub campaign_id: String,
    pub headline: String,
    pub body: String,
    pub cta: Option<String>,
    pub budget_cents: Option<i64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SendEmailInput {
    pub campaign_id: String,
    pub audience_id: String,
    pub subject: String,
    pub body: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AllocateBudgetInput {
    pub campaign_id: String,
    pub allocations: serde_json::Value, // {"google": 5000, "meta": 3000, "email": 1000}
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DateRangeInput { pub campaign_id: Option<String>, pub start_date: Option<String>, pub end_date: Option<String> }

#[derive(Clone)]
pub struct MarketingServer { pub api: ApiClient }

fn r(result: Result<serde_json::Value, anyhow::Error>) -> String {
    match result {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
        Err(e) => format!("Error: {}", e),
    }
}

#[tool_router(server_handler)]
impl MarketingServer {
    // === Campaigns (5) ===

    #[tool(description = "List campaigns filtered by status (draft, active, paused, completed) or channel")]
    async fn list_campaigns(&self, Parameters(input): Parameters<FilterInput>) -> String {
        let mut path = "/campaigns?".to_string();
        if let Some(s) = &input.status { path.push_str(&format!("status={}&", s)); }
        if let Some(c) = &input.channel { path.push_str(&format!("channel={}&", c)); }
        r(self.api.get(&path).await)
    }

    #[tool(description = "Get campaign details: channels, audience, budget, performance summary")]
    async fn get_campaign(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.api.get(&format!("/campaigns/{}", input.id)).await)
    }

    #[tool(description = "Create a multi-channel campaign (draft state)")]
    async fn create_campaign(&self, Parameters(input): Parameters<CreateCampaignInput>) -> String {
        r(self.api.post("/campaigns", &json!({
            "name": input.name, "channels": input.channels, "audience_id": input.audience_id,
            "budget_cents": input.budget_cents, "start_date": input.start_date, "end_date": input.end_date,
            "status": "draft"
        })).await)
    }

    #[tool(description = "Launch a draft campaign — activates across all configured channels")]
    async fn launch_campaign(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.api.patch(&format!("/campaigns/{}", input.id), &json!({"status": "active"})).await)
    }

    #[tool(description = "Pause a running campaign")]
    async fn pause_campaign(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.api.patch(&format!("/campaigns/{}", input.id), &json!({"status": "paused"})).await)
    }

    // === Audiences (4) ===

    #[tool(description = "List audience segments with sizes")]
    async fn list_audiences(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        r(self.api.get("/audiences").await)
    }

    #[tool(description = "Get audience segment definition and size")]
    async fn get_audience(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.api.get(&format!("/audiences/{}", input.id)).await)
    }

    #[tool(description = "Create an audience segment from targeting criteria")]
    async fn create_audience(&self, Parameters(input): Parameters<CreateAudienceInput>) -> String {
        r(self.api.post("/audiences", &json!({"name": input.name, "criteria": input.criteria})).await)
    }

    #[tool(description = "Estimate reach for targeting criteria before creating audience")]
    async fn estimate_reach(&self, Parameters(input): Parameters<EstimateReachInput>) -> String {
        r(self.api.post("/audiences/estimate", &json!({"criteria": input.criteria})).await)
    }

    // === Content (4) ===

    #[tool(description = "List content assets for a campaign or all")]
    async fn list_content(&self, Parameters(input): Parameters<FilterInput>) -> String {
        let mut path = "/content?".to_string();
        if let Some(c) = &input.channel { path.push_str(&format!("type={}&", c)); }
        r(self.api.get(&path).await)
    }

    #[tool(description = "Generate campaign copy (email subject, ad text, social post)")]
    async fn create_content(&self, Parameters(input): Parameters<CreateContentInput>) -> String {
        r(self.api.post("/content/generate", &json!({
            "campaign_id": input.campaign_id, "content_type": input.content_type,
            "brief": input.brief, "tone": input.tone.unwrap_or("professional".into())
        })).await)
    }

    #[tool(description = "Get A/B test variants and their performance")]
    async fn get_ab_variants(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.api.get(&format!("/content/{}/variants", input.id)).await)
    }

    #[tool(description = "View content calendar — scheduled posts and emails")]
    async fn get_content_calendar(&self, Parameters(input): Parameters<DateRangeInput>) -> String {
        let mut path = "/content/calendar?".to_string();
        if let Some(s) = &input.start_date { path.push_str(&format!("start={}&", s)); }
        if let Some(e) = &input.end_date { path.push_str(&format!("end={}&", e)); }
        r(self.api.get(&path).await)
    }

    // === Channels (4) ===

    #[tool(description = "List configured marketing channels and their status")]
    async fn list_channels(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        r(self.api.get("/channels").await)
    }

    #[tool(description = "Schedule a social media post")]
    async fn schedule_post(&self, Parameters(input): Parameters<SchedulePostInput>) -> String {
        r(self.api.post("/channels/social/posts", &json!({
            "channel": input.channel, "content": input.content,
            "scheduled_at": input.scheduled_at, "audience_id": input.audience_id
        })).await)
    }

    #[tool(description = "Create an ad on Google, Meta, or LinkedIn")]
    async fn create_ad(&self, Parameters(input): Parameters<CreateAdInput>) -> String {
        r(self.api.post("/channels/ads", &json!({
            "platform": input.platform, "campaign_id": input.campaign_id,
            "headline": input.headline, "body": input.body,
            "cta": input.cta, "budget_cents": input.budget_cents
        })).await)
    }

    #[tool(description = "Send a campaign email to an audience segment")]
    async fn send_campaign_email(&self, Parameters(input): Parameters<SendEmailInput>) -> String {
        r(self.api.post("/channels/email/send", &json!({
            "campaign_id": input.campaign_id, "audience_id": input.audience_id,
            "subject": input.subject, "body": input.body
        })).await)
    }

    // === Performance (3) ===

    #[tool(description = "Get campaign metrics: impressions, clicks, conversions, spend, ROAS")]
    async fn get_campaign_metrics(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.api.get(&format!("/campaigns/{}/metrics", input.id)).await)
    }

    #[tool(description = "Get multi-touch attribution report")]
    async fn get_attribution(&self, Parameters(input): Parameters<DateRangeInput>) -> String {
        let mut path = "/analytics/attribution?".to_string();
        if let Some(c) = &input.campaign_id { path.push_str(&format!("campaign_id={}&", c)); }
        r(self.api.get(&path).await)
    }

    #[tool(description = "Compare performance across channels (email vs ads vs social)")]
    async fn get_channel_comparison(&self, Parameters(input): Parameters<DateRangeInput>) -> String {
        let mut path = "/analytics/channels?".to_string();
        if let Some(c) = &input.campaign_id { path.push_str(&format!("campaign_id={}&", c)); }
        r(self.api.get(&path).await)
    }

    // === Budget (2) ===

    #[tool(description = "Get budget status: spend vs budget, burn rate, forecast")]
    async fn get_budget_status(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.api.get(&format!("/campaigns/{}/budget", input.id)).await)
    }

    #[tool(description = "Allocate/shift budget between channels within a campaign")]
    async fn allocate_budget(&self, Parameters(input): Parameters<AllocateBudgetInput>) -> String {
        r(self.api.post(&format!("/campaigns/{}/budget/allocate", input.campaign_id), &json!({"allocations": input.allocations})).await)
    }
}
