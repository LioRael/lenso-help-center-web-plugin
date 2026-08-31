//! Public, job-first Help Center surface for one configured organization.
//!
//! This Plugin owns only HTTP presentation and orchestration. Published article
//! truth stays in `lenso.knowledge-base@1`, authenticated requester identity
//! stays in `lenso.auth@1`, and support case/attachment truth stays in the
//! respective Support Capabilities.

mod assets;

use std::fmt;

use lenso::prelude::*;
use lenso_auth_sdk::{
    ActorAssertion, ActorAssertionVerifier, ActorProjectionError, AssertionClock,
    AssertionValidationError, AuthOutcome, CredentialEvidence, TypedActor, authenticate_request,
    decode_auth_response,
};
use lenso_capability_auth as auth;
use lenso_capability_http_endpoint::{
    self as http_endpoint_contract, EndpointHandleInvocationError, ExtractorFuture,
    ExtractorRejection, FromRequest, HandleRequest, HandleResponse, HandleResponseHeadersItem,
    Json, Path, endpoint,
    response::{self, HeaderValue, StatusCode, header},
};
use lenso_capability_knowledge_base as knowledge_base;
use lenso_capability_support_attachment as attachment;
use lenso_capability_support_intake as intake;
use lenso_kernel::{InvocationContext, RuntimeFailure};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

const MAX_ORGANIZATION_BYTES: usize = 512;
const MAX_QUERY_BYTES: usize = 240;
const MAX_ARTICLE_REF_BYTES: usize = 240;
const MAX_TITLE_BYTES: usize = 240;
const MAX_DESCRIPTION_BYTES: usize = 12_000;
const MAX_IDEMPOTENCY_KEY_BYTES: usize = 180;
const MAX_CASE_REF_BYTES: usize = 240;
const MAX_MESSAGE_ID_BYTES: usize = 240;
const MAX_FILENAME_BYTES: usize = 255;
const MAX_ATTACHMENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_AUTH_ISSUER_BYTES: usize = 256;
const MAX_AUTH_PUBLIC_KEY_BYTES: usize = 4096;
const CREATE_SUPPORT_AUDIENCE: &str = "help.center.web.support.create";
const SUPPORT_STATUS_AUDIENCE: &str = "help.center.web.support.status";
const UPLOAD_ATTACHMENT_AUDIENCE: &str = "help.center.web.support.attachment.upload";

/// Forces this native Plugin crate to be retained by a linked Host.
pub const fn link() {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, lenso::PluginConfig)]
#[serde(deny_unknown_fields)]
pub struct HelpCenterConfig {
    organization_id: String,
    auth_issuer: String,
    auth_assertion_public_key: String,
}

impl HelpCenterConfig {
    pub fn new(
        organization_id: impl Into<String>,
        auth_issuer: impl Into<String>,
        auth_assertion_public_key: impl Into<String>,
    ) -> Result<Self, RuntimeFailure> {
        let config = Self {
            organization_id: organization_id.into(),
            auth_issuer: auth_issuer.into(),
            auth_assertion_public_key: auth_assertion_public_key.into(),
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), RuntimeFailure> {
        if !valid_opaque_identifier(&self.organization_id, MAX_ORGANIZATION_BYTES) {
            return Err(RuntimeFailure::InvalidResolvedPlan {
                detail: "Help Center organization_id is invalid".to_owned(),
            });
        }
        if !valid_opaque_identifier(&self.auth_issuer, MAX_AUTH_ISSUER_BYTES) {
            return Err(RuntimeFailure::InvalidResolvedPlan {
                detail: "Help Center auth_issuer is invalid".to_owned(),
            });
        }
        if !(1..=MAX_AUTH_PUBLIC_KEY_BYTES).contains(&self.auth_assertion_public_key.len())
            || ActorAssertionVerifier::from_public_key_base64(
                self.auth_issuer.clone(),
                &self.auth_assertion_public_key,
            )
            .is_err()
        {
            return Err(RuntimeFailure::InvalidResolvedPlan {
                detail: "Help Center Auth assertion public key is invalid".to_owned(),
            });
        }
        Ok(())
    }

    fn verifier(&self) -> Result<ActorAssertionVerifier, RuntimeFailure> {
        ActorAssertionVerifier::from_public_key_base64(
            self.auth_issuer.clone(),
            &self.auth_assertion_public_key,
        )
        .map_err(|_| RuntimeFailure::InvalidResolvedPlan {
            detail: "Help Center Auth assertion public key is invalid".to_owned(),
        })
    }
}

fn validate_config(config: &HelpCenterConfig) -> Result<(), RuntimeFailure> {
    config.validate()
}

#[lenso::plugin(validate = validate_config)]
#[derive(Clone, Debug)]
pub struct HelpCenterWebPlugin {
    #[config]
    config: HelpCenterConfig,
    auth: Port<auth::AuthClient>,
    knowledge_base: Port<knowledge_base::KnowledgeBaseClient>,
    intake: Port<intake::SupportIntakeClient>,
    attachment: Port<attachment::SupportAttachmentClient>,
}

#[endpoint]
impl HelpCenterWebPlugin {
    #[get("help.center.web.page", "/help")]
    async fn page(&self) -> Result<HandleResponse, EndpointHandleInvocationError> {
        std::future::ready(()).await;
        Ok(asset(
            StatusCode::OK,
            "text/html; charset=utf-8",
            assets::PAGE,
        ))
    }

    #[get("help.center.web.css", "/help/assets/app.css")]
    async fn css(&self) -> Result<HandleResponse, EndpointHandleInvocationError> {
        std::future::ready(()).await;
        Ok(asset(
            StatusCode::OK,
            "text/css; charset=utf-8",
            assets::CSS,
        ))
    }

    #[get("help.center.web.js", "/help/assets/app.js")]
    async fn javascript(&self) -> Result<HandleResponse, EndpointHandleInvocationError> {
        std::future::ready(()).await;
        Ok(asset(
            StatusCode::OK,
            "text/javascript; charset=utf-8",
            assets::JS,
        ))
    }

    #[post("help.center.web.articles.search", "/api/help/articles/search")]
    async fn search_articles(
        &self,
        context: InvocationContext,
        Json(input): Json<SearchInput>,
    ) -> Result<HandleResponse, EndpointHandleInvocationError> {
        let query = match bounded_text(&input.query, MAX_QUERY_BYTES, "query") {
            Ok(value) => value,
            Err(problem) => return Ok(problem),
        };
        if !(1..=20).contains(&input.limit) {
            return Ok(invalid_request(
                "invalid_limit",
                "limit must be between 1 and 20.",
            ));
        }

        match self
            .knowledge_base
            .search_published_articles_with_context(
                context,
                knowledge_base::SearchPublishedArticlesRequest {
                    limit: input.limit,
                    organization_id: self.config.organization_id.clone(),
                    query,
                },
            )
            .await
        {
            Ok(result) => json(
                StatusCode::OK,
                &ArticleSearchResults {
                    index_revision: result.index_revision,
                    articles: result
                        .articles
                        .into_iter()
                        .map(|article| ArticleSummary {
                            article_ref: article.slug,
                            title: article.title,
                            published_at: article.published_at,
                        })
                        .collect(),
                },
            ),
            Err(error) => map_search_error(error),
        }
    }

    #[get("help.center.web.articles.get", "/api/help/articles/{article_ref}")]
    async fn get_article(
        &self,
        context: InvocationContext,
        Path(path): Path<ArticlePath>,
    ) -> Result<HandleResponse, EndpointHandleInvocationError> {
        let article_ref =
            match valid_reference(&path.article_ref, MAX_ARTICLE_REF_BYTES, "article_ref") {
                Ok(value) => value,
                Err(problem) => return Ok(problem),
            };

        match self
            .knowledge_base
            .get_published_article_with_context(
                context,
                knowledge_base::GetPublishedArticleRequest {
                    article_ref,
                    organization_id: self.config.organization_id.clone(),
                },
            )
            .await
        {
            Ok(article) => json(
                StatusCode::OK,
                &ArticleView {
                    article_ref: article.slug,
                    title: article.title,
                    body_markdown: article.body_markdown,
                    published_at: article.published_at,
                },
            ),
            Err(error) => map_get_article_error(error),
        }
    }

    #[post("help.center.web.support.create", "/api/help/support")]
    async fn create_support_request(
        &self,
        requester: AuthenticatedRequester,
        context: InvocationContext,
        Json(input): Json<CreateSupportRequest>,
    ) -> Result<HandleResponse, EndpointHandleInvocationError> {
        let (requester, context) =
            match requester.authorize(&self.config, context, CREATE_SUPPORT_AUDIENCE) {
                Ok(authorized) => authorized,
                Err(RequesterAuthorizationFailure::Response(response)) => return Ok(response),
                Err(RequesterAuthorizationFailure::Runtime(error)) => {
                    return Err(EndpointHandleInvocationError::Runtime(error));
                }
            };
        let input = match input.validate() {
            Ok(value) => value,
            Err(problem) => return Ok(problem),
        };

        match self
            .intake
            .open_case_from_channel_with_context(
                context,
                intake::OpenCaseFromChannelRequest {
                    description: input.description,
                    idempotency_key: prefixed_idempotency("help-case", &input.idempotency_key),
                    organization_id: self.config.organization_id.clone(),
                    priority: intake::OpenCaseFromChannelRequestPriority::Normal,
                    requester_subject: requester.subject,
                    title: input.title,
                },
            )
            .await
        {
            Ok(case) => json(
                StatusCode::CREATED,
                &SupportReceipt {
                    case_id: case.case_id,
                    case_ref: case.identifier,
                    title: case.title,
                    state: open_state(&case.state),
                    created_at: case.created_at,
                },
            ),
            Err(error) => map_open_case_error(error),
        }
    }

    #[post("help.center.web.support.status", "/api/help/support/status")]
    async fn support_status(
        &self,
        requester: AuthenticatedRequester,
        context: InvocationContext,
        Json(input): Json<SupportStatusRequest>,
    ) -> Result<HandleResponse, EndpointHandleInvocationError> {
        let (requester, context) =
            match requester.authorize(&self.config, context, SUPPORT_STATUS_AUDIENCE) {
                Ok(authorized) => authorized,
                Err(RequesterAuthorizationFailure::Response(response)) => return Ok(response),
                Err(RequesterAuthorizationFailure::Runtime(error)) => {
                    return Err(EndpointHandleInvocationError::Runtime(error));
                }
            };
        let input = match input.validate() {
            Ok(value) => value,
            Err(problem) => return Ok(problem),
        };
        let requester_subject = requester.subject;
        let case = match self
            .intake
            .get_requester_case_with_context(
                context.clone(),
                intake::GetRequesterCaseRequest {
                    case_ref: input.case_ref.clone(),
                    organization_id: self.config.organization_id.clone(),
                    requester_subject: requester_subject.clone(),
                },
            )
            .await
        {
            Ok(value) => value,
            Err(error) => return map_get_case_error(error),
        };
        let messages = match self
            .intake
            .list_requester_messages_with_context(
                context,
                intake::ListRequesterMessagesRequest {
                    case_ref: input.case_ref,
                    cursor: None,
                    limit: 100,
                    organization_id: self.config.organization_id.clone(),
                    requester_subject: requester_subject.clone(),
                },
            )
            .await
        {
            Ok(value) => value,
            Err(error) => return map_list_messages_error(error),
        };

        json(
            StatusCode::OK,
            &SupportStatus {
                case_id: case.case_id,
                case_ref: case.identifier,
                title: case.title,
                description: case.description,
                state: requester_state(&case.state),
                updated_at: case.updated_at,
                messages: messages
                    .messages
                    .into_iter()
                    .map(|message| PublicMessage {
                        author: if message.author_subject == requester_subject {
                            "you"
                        } else {
                            "support"
                        },
                        body: message.body,
                        created_at: message.created_at,
                    })
                    .collect(),
                next_cursor: messages.next_cursor,
            },
        )
    }

    #[post(
        "help.center.web.support.attachment.upload",
        "/api/help/support/attachments"
    )]
    async fn upload_attachment(
        &self,
        requester: AuthenticatedRequester,
        context: InvocationContext,
        Json(input): Json<UploadAttachmentRequest>,
    ) -> Result<HandleResponse, EndpointHandleInvocationError> {
        let (_requester, context) =
            match requester.authorize(&self.config, context, UPLOAD_ATTACHMENT_AUDIENCE) {
                Ok(authorized) => authorized,
                Err(RequesterAuthorizationFailure::Response(response)) => return Ok(response),
                Err(RequesterAuthorizationFailure::Runtime(error)) => {
                    return Err(EndpointHandleInvocationError::Runtime(error));
                }
            };
        let input = match input.validate() {
            Ok(value) => value,
            Err(problem) => return Ok(problem),
        };
        match self
            .attachment
            .upload_and_attach_with_context(
                context,
                attachment::UploadAndAttachRequest {
                    case_ref: input.case_ref,
                    content: input.content,
                    content_type: input.content_type,
                    filename: input.filename,
                    idempotency_key: prefixed_idempotency(
                        "help-attachment",
                        &input.idempotency_key,
                    ),
                    message_id: input.message_id,
                    organization_id: self.config.organization_id.clone(),
                    visibility: input.visibility,
                },
            )
            .await
        {
            Ok(result) => json(
                StatusCode::CREATED,
                &AttachmentReceipt {
                    attachment_id: result.attachment.attachment_id,
                    filename: result.attachment.filename,
                    size_bytes: result.attachment.size_bytes,
                    created_at: result.attachment.created_at,
                    replayed: result.replayed,
                },
            ),
            Err(error) => map_upload_attachment_error(error),
        }
    }
}

#[derive(Debug)]
struct AuthenticatedRequester {
    assertion: ActorAssertion,
}

impl AuthenticatedRequester {
    fn authorize(
        self,
        config: &HelpCenterConfig,
        context: InvocationContext,
        operation: &str,
    ) -> Result<(HelpCenterRequester, InvocationContext), RequesterAuthorizationFailure> {
        let context = self.assertion.attach(context).map_err(|error| {
            RequesterAuthorizationFailure::Runtime(RuntimeFailure::Internal {
                detail: format!("could not attach authenticated actor assertion: {error}"),
            })
        })?;
        let requester = config
            .verifier()
            .map_err(RequesterAuthorizationFailure::Runtime)?
            .project_context::<HelpCenterRequester>(
                &context,
                http_endpoint_contract::CAPABILITY_ID,
                operation,
                &UtcClock,
            )
            .map_err(|error| match error {
                ActorProjectionError::Assertion(AssertionValidationError::AudienceMismatch {
                    ..
                })
                | ActorProjectionError::UnexpectedActorKind { .. } => {
                    RequesterAuthorizationFailure::Response(response::problem(
                        StatusCode::FORBIDDEN,
                        "insufficient_audience",
                        "The authenticated credential is not authorized for this support operation.",
                    ))
                }
                ActorProjectionError::Assertion(_) => {
                    RequesterAuthorizationFailure::Response(authentication_problem())
                }
            })?;
        Ok((requester, context))
    }
}

#[derive(Debug)]
struct HelpCenterRequester {
    subject: String,
}

impl TypedActor for HelpCenterRequester {
    fn from_assertion(assertion: &ActorAssertion) -> Result<Self, ActorProjectionError> {
        if assertion.actor_kind() != "user" {
            return Err(ActorProjectionError::UnexpectedActorKind {
                expected: "user".to_owned(),
                actual: assertion.actor_kind().to_owned(),
            });
        }
        Ok(Self {
            subject: assertion.subject().to_owned(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct UtcClock;

impl AssertionClock for UtcClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

#[derive(Debug)]
enum RequesterAuthorizationFailure {
    Response(HandleResponse),
    Runtime(RuntimeFailure),
}

impl FromRequest<HelpCenterWebPlugin> for AuthenticatedRequester {
    fn from_request<'a>(
        provider: &'a HelpCenterWebPlugin,
        context: &'a mut InvocationContext,
        request: &'a HandleRequest,
    ) -> ExtractorFuture<'a, Self> {
        Box::pin(async move {
            let evidence = request
                .credential
                .as_ref()
                .map(|credential| CredentialEvidence::new(&credential.scheme, &credential.value));
            let response = provider
                .auth
                .authenticate_with_context(context.clone(), authenticate_request(evidence))
                .await
                .map_err(|error| -> ExtractorRejection {
                    match error {
                        auth::AuthInvocationError::Domain(_) => authentication_problem().into(),
                        auth::AuthInvocationError::Runtime(error) => {
                            EndpointHandleInvocationError::Runtime(error).into()
                        }
                    }
                })?;
            let outcome = decode_auth_response(response).map_err(|_| {
                EndpointHandleInvocationError::Runtime(RuntimeFailure::ProtocolViolation {
                    capability: auth::CAPABILITY_ID,
                })
            })?;
            let AuthOutcome::Authenticated(assertion) = outcome else {
                return Err(authentication_problem().into());
            };
            Ok(Self { assertion })
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SearchInput {
    query: String,
    #[serde(default = "default_search_limit")]
    limit: i64,
}

const fn default_search_limit() -> i64 {
    8
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArticlePath {
    article_ref: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CreateSupportRequest {
    title: String,
    description: String,
    idempotency_key: String,
}

impl fmt::Debug for CreateSupportRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateSupportRequest")
            .field("title", &"<redacted>")
            .field("description", &"<redacted>")
            .field("idempotency_key", &self.idempotency_key)
            .finish()
    }
}

impl CreateSupportRequest {
    fn validate(self) -> Result<Self, HandleResponse> {
        let title = bounded_text(&self.title, MAX_TITLE_BYTES, "title")?;
        let description = bounded_text(&self.description, MAX_DESCRIPTION_BYTES, "description")?;
        let idempotency_key = valid_idempotency_key(&self.idempotency_key)?;
        Ok(Self {
            title,
            description,
            idempotency_key,
        })
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SupportStatusRequest {
    case_ref: String,
}

impl fmt::Debug for SupportStatusRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SupportStatusRequest")
            .field("case_ref", &self.case_ref)
            .finish()
    }
}

impl SupportStatusRequest {
    fn validate(self) -> Result<Self, HandleResponse> {
        Ok(Self {
            case_ref: valid_reference(&self.case_ref, MAX_CASE_REF_BYTES, "case_ref")?,
        })
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UploadAttachmentRequest {
    case_ref: String,
    #[serde(default)]
    message_id: Option<String>,
    filename: String,
    content_type: attachment::UploadContentType,
    content: attachment::Bytes,
    visibility: attachment::Visibility,
    idempotency_key: String,
}

impl fmt::Debug for UploadAttachmentRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UploadAttachmentRequest")
            .field("case_ref", &self.case_ref)
            .field("message_id", &self.message_id)
            .field("filename", &self.filename)
            .field("content_type", &self.content_type)
            .field("content", &"<redacted>")
            .field("visibility", &self.visibility)
            .field("idempotency_key", &self.idempotency_key)
            .finish()
    }
}

impl UploadAttachmentRequest {
    fn validate(self) -> Result<Self, HandleResponse> {
        let case_ref = valid_reference(&self.case_ref, MAX_CASE_REF_BYTES, "case_ref")?;
        let message_id =
            normalized_optional_reference(self.message_id, MAX_MESSAGE_ID_BYTES, "message_id")?;
        let filename = valid_filename(&self.filename)?;
        if !matches!(self.content_type, attachment::UploadContentType::TextPlain) {
            return Err(invalid_request(
                "unsupported_content_type",
                "Only text/plain attachments are supported.",
            ));
        }
        if !matches!(self.visibility, attachment::Visibility::Public) {
            return Err(invalid_request(
                "unsupported_visibility",
                "Requester uploads must have public visibility.",
            ));
        }
        if self.content.is_empty()
            || self.content.len() > MAX_ATTACHMENT_BYTES
            || std::str::from_utf8(self.content.as_slice()).is_err()
            || self.content.as_slice().contains(&0)
        {
            return Err(invalid_request(
                "invalid_attachment_content",
                "Attachment content must be non-empty UTF-8 text/plain and at most 8 MiB.",
            ));
        }
        Ok(Self {
            case_ref,
            message_id,
            filename,
            content_type: self.content_type,
            content: self.content,
            visibility: self.visibility,
            idempotency_key: valid_idempotency_key(&self.idempotency_key)?,
        })
    }
}

#[derive(Debug, Serialize)]
struct ArticleSearchResults {
    index_revision: String,
    articles: Vec<ArticleSummary>,
}

#[derive(Debug, Serialize)]
struct ArticleSummary {
    article_ref: String,
    title: String,
    published_at: String,
}

#[derive(Debug, Serialize)]
struct ArticleView {
    article_ref: String,
    title: String,
    body_markdown: String,
    published_at: String,
}

#[derive(Debug, Serialize)]
struct SupportReceipt {
    case_id: String,
    case_ref: String,
    title: String,
    state: &'static str,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct SupportStatus {
    case_id: String,
    case_ref: String,
    title: String,
    description: String,
    state: &'static str,
    updated_at: String,
    messages: Vec<PublicMessage>,
    next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
struct AttachmentReceipt {
    attachment_id: String,
    filename: String,
    size_bytes: i64,
    created_at: String,
    replayed: bool,
}

#[derive(Debug, Serialize)]
struct PublicMessage {
    author: &'static str,
    body: String,
    created_at: String,
}

fn bounded_text(value: &str, max_bytes: usize, field: &str) -> Result<String, HandleResponse> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > max_bytes
        || value.chars().any(|character| character == '\0')
    {
        return Err(invalid_request(
            &format!("invalid_{field}"),
            format!("{field} must be non-empty and at most {max_bytes} UTF-8 bytes."),
        ));
    }
    Ok(value.to_owned())
}

fn normalized_optional_reference(
    value: Option<String>,
    max_bytes: usize,
    field: &str,
) -> Result<Option<String>, HandleResponse> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    valid_reference(value, max_bytes, field).map(Some)
}

fn valid_filename(value: &str) -> Result<String, HandleResponse> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > MAX_FILENAME_BYTES
        || matches!(value, "." | "..")
        || value
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'))
    {
        return Err(invalid_request(
            "invalid_filename",
            "filename must be one basename of at most 255 UTF-8 bytes.",
        ));
    }
    Ok(value.to_owned())
}

fn valid_idempotency_key(value: &str) -> Result<String, HandleResponse> {
    let value = value.trim();
    if !valid_opaque_identifier(value, MAX_IDEMPOTENCY_KEY_BYTES) {
        return Err(invalid_request(
            "invalid_idempotency_key",
            "idempotency_key must contain only letters, digits, '.', '_', ':', or '-'.",
        ));
    }
    Ok(value.to_owned())
}

fn valid_reference(value: &str, max_bytes: usize, field: &str) -> Result<String, HandleResponse> {
    let value = value.trim();
    if !valid_opaque_identifier(value, max_bytes) {
        return Err(invalid_request(
            &format!("invalid_{field}"),
            format!("{field} contains unsupported characters."),
        ));
    }
    Ok(value.to_owned())
}

fn valid_opaque_identifier(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn prefixed_idempotency(prefix: &str, value: &str) -> String {
    format!("{prefix}:{value}")
}

fn open_state(state: &intake::OpenCaseFromChannelResponseState) -> &'static str {
    match state {
        intake::OpenCaseFromChannelResponseState::Open => "open",
        intake::OpenCaseFromChannelResponseState::InProgress => "in_progress",
        intake::OpenCaseFromChannelResponseState::WaitingCustomer => "waiting_customer",
        intake::OpenCaseFromChannelResponseState::Resolved => "resolved",
        intake::OpenCaseFromChannelResponseState::Closed => "closed",
    }
}

fn requester_state(state: &intake::GetRequesterCaseResponseState) -> &'static str {
    match state {
        intake::GetRequesterCaseResponseState::Open => "open",
        intake::GetRequesterCaseResponseState::InProgress => "in_progress",
        intake::GetRequesterCaseResponseState::WaitingCustomer => "waiting_customer",
        intake::GetRequesterCaseResponseState::Resolved => "resolved",
        intake::GetRequesterCaseResponseState::Closed => "closed",
    }
}

fn asset(status: StatusCode, content_type: &str, body: &str) -> HandleResponse {
    HandleResponse {
        body: body.as_bytes().to_vec().into(),
        headers: vec![HandleResponseHeadersItem {
            name: "content-type".to_owned(),
            value: content_type.to_owned(),
        }],
        status: i64::from(status.as_u16()),
    }
}

fn json<T: Serialize>(
    status: StatusCode,
    value: &T,
) -> Result<HandleResponse, EndpointHandleInvocationError> {
    response::json(status, value).map_err(Into::into)
}

fn invalid_request(code: &str, detail: impl Into<String>) -> HandleResponse {
    response::problem(StatusCode::BAD_REQUEST, code, detail)
}

fn authentication_problem() -> HandleResponse {
    response::problem(
        StatusCode::UNAUTHORIZED,
        "authentication_required",
        "Provide a valid Bearer credential for support operations.",
    )
    .with_header(
        &header::WWW_AUTHENTICATE,
        &HeaderValue::from_static("Bearer"),
    )
    .expect("the static WWW-Authenticate header is valid")
}

fn unavailable(detail: &str) -> HandleResponse {
    response::problem(
        StatusCode::SERVICE_UNAVAILABLE,
        "help_center_unavailable",
        detail,
    )
}

fn protocol_violation(capability: &'static str) -> EndpointHandleInvocationError {
    EndpointHandleInvocationError::Runtime(RuntimeFailure::ProtocolViolation { capability })
}

fn map_search_error(
    error: knowledge_base::KnowledgeBaseSearchPublishedArticlesInvocationError,
) -> Result<HandleResponse, EndpointHandleInvocationError> {
    match error {
        knowledge_base::KnowledgeBaseSearchPublishedArticlesInvocationError::Domain(
            knowledge_base::SearchPublishedArticlesError::InvalidQuery,
        ) => Ok(invalid_request(
            "invalid_query",
            "The Knowledge Base rejected this search query.",
        )),
        knowledge_base::KnowledgeBaseSearchPublishedArticlesInvocationError::Domain(
            knowledge_base::SearchPublishedArticlesError::Forbidden
            | knowledge_base::SearchPublishedArticlesError::Unauthenticated,
        ) => Ok(unavailable("Published help content is not available.")),
        knowledge_base::KnowledgeBaseSearchPublishedArticlesInvocationError::Domain(
            knowledge_base::SearchPublishedArticlesError::Unknown(_),
        ) => Err(protocol_violation(knowledge_base::CAPABILITY_ID)),
        knowledge_base::KnowledgeBaseSearchPublishedArticlesInvocationError::Runtime(error) => {
            Err(EndpointHandleInvocationError::Runtime(error))
        }
    }
}

fn map_get_article_error(
    error: knowledge_base::KnowledgeBaseGetPublishedArticleInvocationError,
) -> Result<HandleResponse, EndpointHandleInvocationError> {
    match error {
        knowledge_base::KnowledgeBaseGetPublishedArticleInvocationError::Domain(
            knowledge_base::GetPublishedArticleError::ArticleNotFound,
        ) => Ok(response::problem(
            StatusCode::NOT_FOUND,
            "article_not_found",
            "That published help article was not found.",
        )),
        knowledge_base::KnowledgeBaseGetPublishedArticleInvocationError::Domain(
            knowledge_base::GetPublishedArticleError::InvalidRequest,
        ) => Ok(invalid_request(
            "invalid_article_ref",
            "The Knowledge Base rejected this article reference.",
        )),
        knowledge_base::KnowledgeBaseGetPublishedArticleInvocationError::Domain(
            knowledge_base::GetPublishedArticleError::Forbidden
            | knowledge_base::GetPublishedArticleError::Unauthenticated,
        ) => Ok(unavailable("Published help content is not available.")),
        knowledge_base::KnowledgeBaseGetPublishedArticleInvocationError::Domain(
            knowledge_base::GetPublishedArticleError::Unknown(_),
        ) => Err(protocol_violation(knowledge_base::CAPABILITY_ID)),
        knowledge_base::KnowledgeBaseGetPublishedArticleInvocationError::Runtime(error) => {
            Err(EndpointHandleInvocationError::Runtime(error))
        }
    }
}

fn map_open_case_error(
    error: intake::SupportIntakeOpenCaseFromChannelInvocationError,
) -> Result<HandleResponse, EndpointHandleInvocationError> {
    match error {
        intake::SupportIntakeOpenCaseFromChannelInvocationError::Domain(
            intake::OpenCaseFromChannelError::InvalidRequest,
        ) => Ok(invalid_request(
            "invalid_support_request",
            "The Support Intake capability rejected this request.",
        )),
        intake::SupportIntakeOpenCaseFromChannelInvocationError::Domain(
            intake::OpenCaseFromChannelError::IdempotencyConflict,
        ) => Ok(response::problem(
            StatusCode::CONFLICT,
            "idempotency_conflict",
            "This idempotency key was already used for a different support request.",
        )),
        intake::SupportIntakeOpenCaseFromChannelInvocationError::Domain(
            intake::OpenCaseFromChannelError::Forbidden,
        ) => Ok(unavailable(
            "Support Intake is not configured for this Help Center.",
        )),
        intake::SupportIntakeOpenCaseFromChannelInvocationError::Domain(
            intake::OpenCaseFromChannelError::Unknown(_),
        ) => Err(protocol_violation(intake::CAPABILITY_ID)),
        intake::SupportIntakeOpenCaseFromChannelInvocationError::Runtime(error) => {
            Err(EndpointHandleInvocationError::Runtime(error))
        }
    }
}

fn map_get_case_error(
    error: intake::SupportIntakeGetRequesterCaseInvocationError,
) -> Result<HandleResponse, EndpointHandleInvocationError> {
    match error {
        intake::SupportIntakeGetRequesterCaseInvocationError::Domain(
            intake::GetRequesterCaseError::CaseNotFound | intake::GetRequesterCaseError::Forbidden,
        ) => Ok(response::problem(
            StatusCode::NOT_FOUND,
            "support_request_not_found",
            "No support request matched this signed-in requester and reference.",
        )),
        intake::SupportIntakeGetRequesterCaseInvocationError::Domain(
            intake::GetRequesterCaseError::InvalidRequest,
        ) => Ok(invalid_request(
            "invalid_support_reference",
            "The Support Intake capability rejected this request reference.",
        )),
        intake::SupportIntakeGetRequesterCaseInvocationError::Domain(
            intake::GetRequesterCaseError::Unknown(_),
        ) => Err(protocol_violation(intake::CAPABILITY_ID)),
        intake::SupportIntakeGetRequesterCaseInvocationError::Runtime(error) => {
            Err(EndpointHandleInvocationError::Runtime(error))
        }
    }
}

fn map_list_messages_error(
    error: intake::SupportIntakeListRequesterMessagesInvocationError,
) -> Result<HandleResponse, EndpointHandleInvocationError> {
    match error {
        intake::SupportIntakeListRequesterMessagesInvocationError::Domain(
            intake::ListRequesterMessagesError::CaseNotFound
            | intake::ListRequesterMessagesError::Forbidden,
        ) => Ok(response::problem(
            StatusCode::NOT_FOUND,
            "support_request_not_found",
            "No support request matched this signed-in requester and reference.",
        )),
        intake::SupportIntakeListRequesterMessagesInvocationError::Domain(
            intake::ListRequesterMessagesError::InvalidRequest,
        ) => Ok(invalid_request(
            "invalid_support_reference",
            "The Support Intake capability rejected this request reference.",
        )),
        intake::SupportIntakeListRequesterMessagesInvocationError::Domain(
            intake::ListRequesterMessagesError::Unknown(_),
        ) => Err(protocol_violation(intake::CAPABILITY_ID)),
        intake::SupportIntakeListRequesterMessagesInvocationError::Runtime(error) => {
            Err(EndpointHandleInvocationError::Runtime(error))
        }
    }
}

fn map_upload_attachment_error(
    error: attachment::SupportAttachmentUploadAndAttachInvocationError,
) -> Result<HandleResponse, EndpointHandleInvocationError> {
    match error {
        attachment::SupportAttachmentUploadAndAttachInvocationError::Domain(
            attachment::UploadAndAttachError::ContentRejected
            | attachment::UploadAndAttachError::InvalidRequest,
        ) => Ok(invalid_request(
            "attachment_rejected",
            "The attachment must be valid UTF-8 text/plain and no larger than 8 MiB.",
        )),
        attachment::SupportAttachmentUploadAndAttachInvocationError::Domain(
            attachment::UploadAndAttachError::IdempotencyConflict,
        ) => Ok(response::problem(
            StatusCode::CONFLICT,
            "idempotency_conflict",
            "This idempotency key was already used for a different attachment.",
        )),
        attachment::SupportAttachmentUploadAndAttachInvocationError::Domain(
            attachment::UploadAndAttachError::Forbidden,
        ) => Ok(response::problem(
            StatusCode::NOT_FOUND,
            "support_request_not_found",
            "No support request matched this signed-in requester and reference.",
        )),
        attachment::SupportAttachmentUploadAndAttachInvocationError::Domain(
            attachment::UploadAndAttachError::Unauthenticated,
        ) => Ok(authentication_problem()),
        attachment::SupportAttachmentUploadAndAttachInvocationError::Domain(
            attachment::UploadAndAttachError::Unknown(_),
        ) => Err(protocol_violation(attachment::CAPABILITY_ID)),
        attachment::SupportAttachmentUploadAndAttachInvocationError::Runtime(error) => {
            Err(EndpointHandleInvocationError::Runtime(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use lenso_auth_sdk::{ActorAssertionIssuer, Validity, audience};
    use lenso_capability_http_endpoint::testing::EndpointTest;
    use lenso_kernel::CancellationToken;
    use time::Duration;

    use super::*;

    fn auth_issuer() -> ActorAssertionIssuer {
        ActorAssertionIssuer::from_signing_key("support-auth", [7_u8; 32])
    }

    fn config() -> HelpCenterConfig {
        let issuer = auth_issuer();
        HelpCenterConfig::new("org_demo", "support-auth", issuer.public_key_base64()).unwrap()
    }

    fn context() -> InvocationContext {
        InvocationContext::new(1, None, CancellationToken::new())
    }

    fn plugin() -> HelpCenterWebPlugin {
        HelpCenterWebPlugin {
            config: config(),
            auth: Port::default(),
            knowledge_base: Port::default(),
            intake: Port::default(),
            attachment: Port::default(),
        }
    }

    #[test]
    fn serves_the_self_contained_job_first_page_without_opening_a_socket() {
        block_on(async {
            let endpoint = EndpointTest::new(plugin());
            let page = endpoint
                .request("help.center.web.page")
                .send()
                .await
                .unwrap();
            assert_eq!(page.status(), StatusCode::OK);
            assert_eq!(
                page.header("content-type"),
                Some("text/html; charset=utf-8")
            );
            let body = page.into_inner().body;
            assert!(body.starts_with(b"<!doctype html>"));
            assert!(
                body.windows(20)
                    .any(|chunk| chunk == b"Search for an answer")
            );

            let css = endpoint
                .request("help.center.web.css")
                .send()
                .await
                .unwrap();
            assert_eq!(css.header("content-type"), Some("text/css; charset=utf-8"));

            let javascript = endpoint.request("help.center.web.js").send().await.unwrap();
            assert_eq!(
                javascript.header("content-type"),
                Some("text/javascript; charset=utf-8")
            );
            assert!(
                javascript
                    .into_inner()
                    .body
                    .windows(25)
                    .any(|chunk| chunk == b"/api/help/articles/search")
            );
        });
    }

    #[test]
    fn rejects_invalid_public_search_input_before_invoking_a_dependency() {
        block_on(async {
            let endpoint = EndpointTest::new(plugin());
            let search = endpoint
                .request("help.center.web.articles.search")
                .json(&SearchInput {
                    query: "   ".to_owned(),
                    limit: 8,
                })
                .unwrap()
                .send()
                .await
                .unwrap();
            assert_eq!(search.status(), StatusCode::BAD_REQUEST);
        });
    }

    #[test]
    fn descriptor_declares_exact_public_surface_and_dependencies() {
        let descriptor: serde_json::Value = serde_json::from_str(PLUGIN_DESCRIPTOR_JSON).unwrap();
        assert_eq!(descriptor["plugin_id"], "lenso.help-center.web");

        let provided = descriptor["provided_capabilities"].as_array().unwrap();
        assert_eq!(provided.len(), 1);
        assert_eq!(provided[0]["capability_id"], "lenso.http.endpoint@1");
        assert_eq!(provided[0]["descriptor_version"], "1.1.0");

        let mut required = descriptor["required_capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                (
                    entry["capability_id"].as_str().unwrap(),
                    entry["descriptor_version"].as_str().unwrap(),
                    entry["cardinality"].as_str().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        required.sort_unstable();
        assert_eq!(
            required,
            vec![
                ("lenso.auth@1", "1.0.0", "one"),
                ("lenso.knowledge-base@1", "1.1.0", "one"),
                ("lenso.support-attachment@1", "1.1.0", "one"),
                ("lenso.support-intake@1", "1.0.0", "one"),
            ]
        );
        assert_eq!(
            descriptor["configuration_schema"]["required"],
            serde_json::json!([
                "organization_id",
                "auth_issuer",
                "auth_assertion_public_key"
            ])
        );
    }

    #[test]
    fn validates_config_and_redacts_customer_input_debug_output() {
        let issuer = auth_issuer();
        assert!(
            HelpCenterConfig::new("org_demo", "support-auth", issuer.public_key_base64()).is_ok()
        );
        assert!(
            HelpCenterConfig::new("org demo", "support-auth", issuer.public_key_base64()).is_err()
        );
        assert!(HelpCenterConfig::new("org_demo", "support-auth", "not-a-key").is_err());

        let request = CreateSupportRequest {
            title: "Need help".to_owned(),
            description: "private details".to_owned(),
            idempotency_key: "request-2".to_owned(),
        };
        let debug = format!("{request:?}");
        assert!(!debug.contains("Need help"));
        assert!(!debug.contains("private details"));
        assert!(
            CreateSupportRequest {
                title: " ".to_owned(),
                description: "details".to_owned(),
                idempotency_key: "request-3".to_owned(),
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn verifies_proof_expiry_actor_kind_and_exact_route_audience_before_orchestration() {
        let issuer = auth_issuer();
        let now = OffsetDateTime::now_utc();
        let validity =
            Validity::new(now - Duration::seconds(1), now + Duration::minutes(5)).unwrap();

        let wrong_audience = AuthenticatedRequester {
            assertion: issuer.issue(
                "customer-1",
                "user",
                "session",
                [audience("unrelated.capability@1", "read")],
                validity,
                std::collections::BTreeMap::new(),
            ),
        };
        let error = wrong_audience
            .authorize(&config(), context(), CREATE_SUPPORT_AUDIENCE)
            .unwrap_err();
        assert!(matches!(
            error,
            RequesterAuthorizationFailure::Response(HandleResponse { status: 403, .. })
        ));

        let expired = AuthenticatedRequester {
            assertion: issuer.issue(
                "customer-1",
                "user",
                "session",
                [audience(
                    http_endpoint_contract::CAPABILITY_ID,
                    CREATE_SUPPORT_AUDIENCE,
                )],
                Validity::new(now - Duration::minutes(10), now - Duration::minutes(5)).unwrap(),
                std::collections::BTreeMap::new(),
            ),
        };
        let error = expired
            .authorize(&config(), context(), CREATE_SUPPORT_AUDIENCE)
            .unwrap_err();
        assert!(matches!(
            error,
            RequesterAuthorizationFailure::Response(HandleResponse { status: 401, .. })
        ));

        let untrusted_issuer = ActorAssertionIssuer::from_signing_key("support-auth", [8_u8; 32]);
        let invalid_proof = AuthenticatedRequester {
            assertion: untrusted_issuer.issue(
                "customer-1",
                "user",
                "session",
                [audience(
                    http_endpoint_contract::CAPABILITY_ID,
                    CREATE_SUPPORT_AUDIENCE,
                )],
                validity,
                std::collections::BTreeMap::new(),
            ),
        };
        let error = invalid_proof
            .authorize(&config(), context(), CREATE_SUPPORT_AUDIENCE)
            .unwrap_err();
        assert!(matches!(
            error,
            RequesterAuthorizationFailure::Response(HandleResponse { status: 401, .. })
        ));

        let wrong_actor = AuthenticatedRequester {
            assertion: issuer.issue(
                "service-1",
                "service",
                "workload",
                [audience(
                    http_endpoint_contract::CAPABILITY_ID,
                    CREATE_SUPPORT_AUDIENCE,
                )],
                validity,
                std::collections::BTreeMap::new(),
            ),
        };
        let error = wrong_actor
            .authorize(&config(), context(), CREATE_SUPPORT_AUDIENCE)
            .unwrap_err();
        assert!(matches!(
            error,
            RequesterAuthorizationFailure::Response(HandleResponse { status: 403, .. })
        ));

        let valid = AuthenticatedRequester {
            assertion: issuer.issue(
                "customer-1",
                "user",
                "session",
                [audience(
                    http_endpoint_contract::CAPABILITY_ID,
                    CREATE_SUPPORT_AUDIENCE,
                )],
                validity,
                std::collections::BTreeMap::new(),
            ),
        };
        let (requester, _) = valid
            .authorize(&config(), context(), CREATE_SUPPORT_AUDIENCE)
            .unwrap();
        assert_eq!(requester.subject, "customer-1");
    }

    #[test]
    fn requester_attachment_boundary_allows_only_bounded_public_utf8_text() {
        let valid = UploadAttachmentRequest {
            case_ref: "SUP-42".to_owned(),
            message_id: None,
            filename: "diagnostics.log".to_owned(),
            content_type: attachment::UploadContentType::TextPlain,
            content: attachment::Bytes::from(b"bounded log".as_slice()),
            visibility: attachment::Visibility::Public,
            idempotency_key: "upload-1".to_owned(),
        };
        assert!(valid.validate().is_ok());

        let internal = UploadAttachmentRequest {
            case_ref: "SUP-42".to_owned(),
            message_id: None,
            filename: "diagnostics.log".to_owned(),
            content_type: attachment::UploadContentType::TextPlain,
            content: attachment::Bytes::from(b"bounded log".as_slice()),
            visibility: attachment::Visibility::Internal,
            idempotency_key: "upload-2".to_owned(),
        };
        assert!(internal.validate().is_err());
        assert!(valid_filename("../diagnostics.log").is_err());
    }
}
